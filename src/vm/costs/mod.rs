pub mod constants;
pub mod cost_functions;

use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use std::convert::{TryFrom, TryInto};
use std::{cmp, fmt};
use vm::types::{TypeSignature, QualifiedContractIdentifier, NONE};
use vm::{Value, ast, eval_all, InputSize};
use vm::contexts::{Environment, OwnedEnvironment, ContractContext, GlobalContext};
use vm::types::Value::UInt;
use regex::internal::Exec;
use vm::database::MemoryBackingStore;
use chainstate::stacks::boot::boot_code_id;
use vm::costs::cost_functions::{ClarityCostFunction};
use std::collections::{HashMap, BTreeMap};
use vm::errors::{Error, InterpreterResult};

type Result<T> = std::result::Result<T, CostErrors>;

pub const CLARITY_MEMORY_LIMIT: u64 = 100 * 1000 * 1000;

// macro_rules! runtime_cost {
//     ( $cost_spec:expr, $env:expr, $input:expr ) => {{
//         use std::convert::TryInto;
//         use vm::costs::{CostErrors, CostTracker};
//         let input = $input
//             .try_into()
//             .map_err(|_| CostErrors::CostOverflow)
//             .and_then(|input| ($cost_spec).compute_cost(input));
//         match input {
//             Ok(cost) => CostTracker::add_cost($env, cost),
//             Err(e) => Err(e),
//         }
//     }};
// }

// 1. map into input optional, convert to u64
// 2. throw error if can't convert
// 3. add_cost with optional we mapped into
// macro_rules! runtime_cost {
//     ( $cost_function:expr, $env:expr, $input:expr ) => {{
//         use std::convert::TryFrom;
//         use vm::costs::{CostErrors, CostTracker};
//         let cost_result = CostTracker::compute_cost($env, $cost_function, $input);
//         match cost_result  {
//             Ok(cost) => CostTracker::add_cost($env, cost),
//             Err(e) => Err(e),
//         }
//     }};
// }


pub fn runtime_cost<T: Into<InputSize>, C: CostTracker>(
    cost_function: ClarityCostFunction,
    tracker: &mut C,
    input: T) -> Result<()> {

    // input arg of type Option<T> gets converted to an Option<InputSize> and then to an Option<u64>
    let cost_result = tracker.compute_cost(cost_function, input.into().into());

    match cost_result  {
        Ok(cost) => tracker.add_cost(cost),
        Err(e) => Err(e),
    }
}

#[test]
fn test_eval_contract_cost() {
    let mut cost_tracker = LimitedCostTracker::new_max_limit();

    let boot_costs_id = boot_code_id("boot_costs");

    // initialize all cost functions with boot cost contract
    let mut m = HashMap::new();
    for f in ClarityCostFunction::ALL.iter() {
        m.insert(f, ClarityCostFunctionReference::new(boot_costs_id.clone(), f.get_name()));
    }
    cost_tracker.cost_function_references = m;

    // cache boot cost contract
    cost_tracker.cost_contracts.insert(boot_costs_id.clone(), std::include_str!("../../chainstate/stacks/boot/costs.clar"));

    runtime_cost(ClarityCostFunction::StxTransfer, &mut cost_tracker, InputSize::None);
    runtime_cost(ClarityCostFunction::Sub, &mut cost_tracker, 10);

    dbg!(cost_tracker);
}

macro_rules! finally_drop_memory {
    ( $env: expr, $used_mem:expr; $exec:expr ) => {{
        let result = (|| $exec)();
        $env.drop_memory($used_mem);
        result
    }};
}

pub fn analysis_typecheck_cost<T: CostTracker>(
    track: &mut T,
    t1: &TypeSignature,
    t2: &TypeSignature,
) -> Result<()> {
    let t1_size = t1.type_size().map_err(|_| CostErrors::CostOverflow)?;
    let t2_size = t2.type_size().map_err(|_| CostErrors::CostOverflow)?;
    let cost = track.compute_cost(
        ClarityCostFunction::AnalysisTypeCheck,
        Some(cmp::max(t1_size, t2_size) as u64))?;
    track.add_cost(cost)
}

pub trait MemoryConsumer {
    fn get_memory_use(&self) -> u64;
}

impl MemoryConsumer for Value {
    fn get_memory_use(&self) -> u64 {
        self.size().into()
    }
}

pub trait CostTracker {
    fn compute_cost(
        &mut self,
        cost_function: ClarityCostFunction,
        input: Option<u64>) -> Result<ExecutionCost>;
    fn add_cost(&mut self, cost: ExecutionCost) -> Result<()>;
    fn add_memory(&mut self, memory: u64) -> Result<()>;
    fn drop_memory(&mut self, memory: u64);
    fn reset_memory(&mut self);
}

// Don't track!
impl CostTracker for () {
    fn compute_cost(
        &mut self,
        _cost_function: ClarityCostFunction,
        _input: Option<u64>) -> std::result::Result<ExecutionCost, CostErrors> {
        Ok(ExecutionCost::zero())
    }
    fn add_cost(&mut self, _cost: ExecutionCost) -> std::result::Result<(), CostErrors> {
        Ok(())
    }
    fn add_memory(&mut self, _memory: u64) -> std::result::Result<(), CostErrors> {
        Ok(())
    }
    fn drop_memory(&mut self, _memory: u64) {}
    fn reset_memory(&mut self) {}
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ClarityCostFunctionReference {
    pub contract_id: QualifiedContractIdentifier,
    pub function_name: String,
}

impl ClarityCostFunctionReference {
    fn new(id: QualifiedContractIdentifier, name: String) -> ClarityCostFunctionReference {
        ClarityCostFunctionReference { contract_id: id, function_name: name }
    }
}

type ClarityCostContract = &'static str;

#[derive(Debug, Clone, PartialEq)]
pub struct LimitedCostTracker {
    cost_function_references: HashMap<&'static ClarityCostFunction, ClarityCostFunctionReference>,
    cost_contracts: HashMap<QualifiedContractIdentifier, ClarityCostContract>,
    total: ExecutionCost,
    limit: ExecutionCost,
    memory: u64,
    memory_limit: u64,
    free: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CostErrors {
    CostComputationFailed(String),
    CostOverflow,
    CostBalanceExceeded(ExecutionCost, ExecutionCost),
    MemoryBalanceExceeded(u64, u64),
}

impl LimitedCostTracker {
    pub fn new(limit: ExecutionCost) -> LimitedCostTracker {
        LimitedCostTracker {
            cost_function_references: HashMap::new(),
            cost_contracts: HashMap::new(),
            limit,
            memory_limit: CLARITY_MEMORY_LIMIT,
            total: ExecutionCost::zero(),
            memory: 0,
            free: false,
        }
    }
    pub fn new_max_limit() -> LimitedCostTracker {
        LimitedCostTracker {
            cost_function_references: HashMap::new(),
            cost_contracts: HashMap::new(),
            limit: ExecutionCost::max_value(),
            total: ExecutionCost::zero(),
            memory: 0,
            memory_limit: CLARITY_MEMORY_LIMIT,
            free: false,
        }
    }
    pub fn new_free() -> LimitedCostTracker {
        LimitedCostTracker {
            cost_function_references: HashMap::new(),
            cost_contracts: HashMap::new(),
            limit: ExecutionCost::max_value(),
            total: ExecutionCost::zero(),
            memory: 0,
            memory_limit: CLARITY_MEMORY_LIMIT,
            free: true,
        }
    }
    pub fn get_total(&self) -> ExecutionCost {
        self.total.clone()
    }
    pub fn set_total(&mut self, total: ExecutionCost) -> () {
        // used by the miner to "undo" the cost of a transaction when trying to pack a block.
        self.total = total;
    }
}

fn map_gets<K,V>(map: BTreeMap<K,V>, keys: Vec<K>) -> Option<Vec<V>> where K: Ord, V: Clone {
    let mut output: Vec<V> = vec![];
    for k in keys {
        match map.get(&k) {
            Some(v) => output.push(v.clone()),
            None => return None,
        }
    }
    Some(output)
}

fn parse_cost(eval_result: InterpreterResult<Option<Value>>) -> Result<ExecutionCost> {
    match eval_result {
        Ok(Some(Value::Tuple(data))) => {
            let results = (
                data.data_map.get("write_length"),
                data.data_map.get("write_count"),
                data.data_map.get("runtime"),
                data.data_map.get("read_length"),
                data.data_map.get("read_count"),
            );

            match results {
                (Some(UInt(write_length)),
                    Some(UInt(write_count)),
                    Some(UInt(runtime)),
                    Some(UInt(read_length)),
                    Some(UInt(read_count))) => {
                    Ok(ExecutionCost {
                        write_length: (*write_length as u64),
                        write_count: (*write_count as u64),
                        runtime: (*runtime as u64),
                        read_length: (*read_length as u64),
                        read_count: (*read_count as u64),
                    })
                },
                _ => Err(CostErrors::CostComputationFailed("Execution Cost tuple does not contain only UInts".to_string()))
            }
        },
        Ok(Some(_)) => Err(CostErrors::CostComputationFailed("Clarity cost function returned something other than a Cost tuple".to_string())),
        Ok(None) => Err(CostErrors::CostComputationFailed("Clarity cost function returned nothing".to_string())),
        Err(e) => Err(CostErrors::CostComputationFailed(format!("Error evaluating result of cost function: {}", e))),
    }
}

fn compute_cost(
    cost_tracker: &mut LimitedCostTracker,
    cost_function: ClarityCostFunction,
    input_size: Option<u64>) -> Result<ExecutionCost> {

    let mut marf = MemoryBackingStore::new();
    let conn = marf.as_clarity_db();
    let mut global_context = GlobalContext::new(conn, LimitedCostTracker::new_free());

    let cost_function_reference = cost_tracker.cost_function_references
        .get(&cost_function)
        .ok_or(CostErrors::CostComputationFailed("CostFunction not defined".to_string()))?.clone();

    let cost_contract = cost_tracker.cost_contracts
        .get(&cost_function_reference.contract_id)
        .ok_or(CostErrors::CostComputationFailed("Cost Contract not cached".to_string()))?;

    let mut contract_context = ContractContext::new(cost_function_reference.contract_id.clone());

    let program = match input_size {
        Some(size) => format!("{}({} u{})", cost_contract, cost_function_reference.function_name, size),
        None => format!("{}({})", cost_contract, cost_function_reference.function_name)
    };

    let eval_result = global_context.execute(|g| {
        let parsed = ast::build_ast_free(&cost_function_reference.contract_id, program.as_str())?.expressions;
        eval_all(&parsed, &mut contract_context, g)
    });

    parse_cost(eval_result)
}

fn add_cost(
    s: &mut LimitedCostTracker,
    cost: ExecutionCost,
) -> std::result::Result<(), CostErrors> {
    s.total.add(&cost)?;
    if s.total.exceeds(&s.limit) {
        Err(CostErrors::CostBalanceExceeded(
            s.total.clone(),
            s.limit.clone(),
        ))
    } else {
        Ok(())
    }
}

fn add_memory(s: &mut LimitedCostTracker, memory: u64) -> std::result::Result<(), CostErrors> {
    s.memory = s.memory.cost_overflow_add(memory)?;
    if s.memory > s.memory_limit {
        Err(CostErrors::MemoryBalanceExceeded(s.memory, s.memory_limit))
    } else {
        Ok(())
    }
}

fn drop_memory(s: &mut LimitedCostTracker, memory: u64) {
    s.memory = s
        .memory
        .checked_sub(memory)
        .expect("Underflowed dropped memory");
}

impl CostTracker for LimitedCostTracker {
    fn compute_cost(&mut self, cost_function: ClarityCostFunction, input: Option<u64>) -> std::result::Result<ExecutionCost, CostErrors> {
        if self.free { return Ok(ExecutionCost::zero()) }
        compute_cost(self, cost_function, input)
    }
    fn add_cost(&mut self, cost: ExecutionCost) -> std::result::Result<(), CostErrors> {
        if self.free { return Ok(()) }
        add_cost(self, cost)
    }
    fn add_memory(&mut self, memory: u64) -> std::result::Result<(), CostErrors> {
        if self.free { return Ok(()) }
        add_memory(self, memory)
    }
    fn drop_memory(&mut self, memory: u64) {
        if !self.free { drop_memory(self, memory) }
    }
    fn reset_memory(&mut self) {
        if !self.free { self.memory = 0; }
    }
}

impl CostTracker for &mut LimitedCostTracker {
    fn compute_cost(&mut self, cost_function: ClarityCostFunction, input: Option<u64>) -> std::result::Result<ExecutionCost, CostErrors> {
        if self.free { return Ok(ExecutionCost::zero()) }
        compute_cost(self, cost_function, input)
    }
    fn add_cost(&mut self, cost: ExecutionCost) -> std::result::Result<(), CostErrors> {
        if self.free { return Ok(()) }
        add_cost(self, cost)
    }
    fn add_memory(&mut self, memory: u64) -> std::result::Result<(), CostErrors> {
        if self.free { return Ok(()) }
        add_memory(self, memory)
    }
    fn drop_memory(&mut self, memory: u64) {
        if !self.free { drop_memory(self, memory) }
    }
    fn reset_memory(&mut self) {
        if !self.free { self.memory = 0; }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum CostFunctions {
    Constant(u64),
    Linear(u64, u64),
    NLogN(u64, u64),
    LogN(u64, u64),
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct SimpleCostSpecification {
    pub write_count: CostFunctions,
    pub write_length: CostFunctions,
    pub read_count: CostFunctions,
    pub read_length: CostFunctions,
    pub runtime: CostFunctions,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ExecutionCost {
    pub write_length: u64,
    pub write_count: u64,
    pub read_length: u64,
    pub read_count: u64,
    pub runtime: u64,
}

impl fmt::Display for ExecutionCost {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{\"runtime\": {}, \"write_length\": {}, \"write_count\": {}, \"read_length\": {}, \"read_count\": {}}}",
               self.runtime, self.write_length, self.write_count, self.read_length, self.read_count)
    }
}

impl ToSql for ExecutionCost {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput> {
        let val = serde_json::to_string(self).expect("FAIL: could not serialize ExecutionCost");
        Ok(ToSqlOutput::from(val))
    }
}

impl FromSql for ExecutionCost {
    fn column_result(value: ValueRef) -> FromSqlResult<ExecutionCost> {
        let str_val = String::column_result(value)?;
        let parsed = serde_json::from_str(&str_val)
            .expect("CORRUPTION: failed to parse ExecutionCost from DB");
        Ok(parsed)
    }
}

pub trait CostOverflowingMath<T> {
    fn cost_overflow_mul(self, other: T) -> Result<T>;
    fn cost_overflow_add(self, other: T) -> Result<T>;
    fn cost_overflow_sub(self, other: T) -> Result<T>;
}

impl CostOverflowingMath<u64> for u64 {
    fn cost_overflow_mul(self, other: u64) -> Result<u64> {
        self.checked_mul(other)
            .ok_or_else(|| CostErrors::CostOverflow)
    }
    fn cost_overflow_add(self, other: u64) -> Result<u64> {
        self.checked_add(other)
            .ok_or_else(|| CostErrors::CostOverflow)
    }
    fn cost_overflow_sub(self, other: u64) -> Result<u64> {
        self.checked_sub(other)
            .ok_or_else(|| CostErrors::CostOverflow)
    }
}

impl ExecutionCost {
    pub fn zero() -> ExecutionCost {
        Self {
            runtime: 0,
            write_length: 0,
            read_count: 0,
            write_count: 0,
            read_length: 0,
        }
    }

    pub fn max_value() -> ExecutionCost {
        Self {
            runtime: u64::max_value(),
            write_length: u64::max_value(),
            read_count: u64::max_value(),
            write_count: u64::max_value(),
            read_length: u64::max_value(),
        }
    }

    pub fn runtime(runtime: u64) -> ExecutionCost {
        Self {
            runtime,
            write_length: 0,
            read_count: 0,
            write_count: 0,
            read_length: 0,
        }
    }

    pub fn add_runtime(&mut self, runtime: u64) -> Result<()> {
        self.runtime = self.runtime.cost_overflow_add(runtime)?;
        Ok(())
    }

    pub fn add(&mut self, other: &ExecutionCost) -> Result<()> {
        self.runtime = self.runtime.cost_overflow_add(other.runtime)?;
        self.read_count = self.read_count.cost_overflow_add(other.read_count)?;
        self.read_length = self.read_length.cost_overflow_add(other.read_length)?;
        self.write_length = self.write_length.cost_overflow_add(other.write_length)?;
        self.write_count = self.write_count.cost_overflow_add(other.write_count)?;
        Ok(())
    }

    pub fn sub(&mut self, other: &ExecutionCost) -> Result<()> {
        self.runtime = self.runtime.cost_overflow_sub(other.runtime)?;
        self.read_count = self.read_count.cost_overflow_sub(other.read_count)?;
        self.read_length = self.read_length.cost_overflow_sub(other.read_length)?;
        self.write_length = self.write_length.cost_overflow_sub(other.write_length)?;
        self.write_count = self.write_count.cost_overflow_sub(other.write_count)?;
        Ok(())
    }

    pub fn multiply(&mut self, times: u64) -> Result<()> {
        self.runtime = self.runtime.cost_overflow_mul(times)?;
        self.read_count = self.read_count.cost_overflow_mul(times)?;
        self.read_length = self.read_length.cost_overflow_mul(times)?;
        self.write_length = self.write_length.cost_overflow_mul(times)?;
        self.write_count = self.write_count.cost_overflow_mul(times)?;
        Ok(())
    }

    /// Returns whether or not this cost exceeds any dimension of the
    ///  other cost.
    pub fn exceeds(&self, other: &ExecutionCost) -> bool {
        self.runtime > other.runtime
            || self.write_length > other.write_length
            || self.write_count > other.write_count
            || self.read_count > other.read_count
            || self.read_length > other.read_length
    }

    pub fn max_cost(first: ExecutionCost, second: ExecutionCost) -> ExecutionCost {
        Self {
            runtime: first.runtime.max(second.runtime),
            write_length: first.write_length.max(second.write_length),
            write_count: first.write_count.max(second.write_count),
            read_count: first.read_count.max(second.read_count),
            read_length: first.read_length.max(second.read_length),
        }
    }
}

// ONLY WORKS IF INPUT IS u64
fn int_log2(input: u64) -> Option<u64> {
    63_u32.checked_sub(input.leading_zeros()).map(|floor_log| {
        if input.trailing_zeros() == floor_log {
            u64::from(floor_log)
        } else {
            u64::from(floor_log + 1)
        }
    })
}

impl CostFunctions {
    pub fn compute_cost(&self, input: u64) -> Result<u64> {
        match self {
            CostFunctions::Constant(val) => Ok(*val),
            CostFunctions::Linear(a, b) => a.cost_overflow_mul(input)?.cost_overflow_add(*b),
            CostFunctions::LogN(a, b) => {
                // a*log(input)) + b
                //  and don't do log(0).
                int_log2(cmp::max(input, 1))
                    .ok_or_else(|| CostErrors::CostOverflow)?
                    .cost_overflow_mul(*a)?
                    .cost_overflow_add(*b)
            }
            CostFunctions::NLogN(a, b) => {
                // a*input*log(input)) + b
                //  and don't do log(0).
                int_log2(cmp::max(input, 1))
                    .ok_or_else(|| CostErrors::CostOverflow)?
                    .cost_overflow_mul(input)?
                    .cost_overflow_mul(*a)?
                    .cost_overflow_add(*b)
            }
        }
    }
}

impl SimpleCostSpecification {
    pub fn compute_cost(&self, input: u64) -> Result<ExecutionCost> {
        Ok(ExecutionCost {
            write_length: self.write_length.compute_cost(input)?,
            write_count: self.write_count.compute_cost(input)?,
            read_count: self.read_count.compute_cost(input)?,
            read_length: self.read_length.compute_cost(input)?,
            runtime: self.runtime.compute_cost(input)?,
        })
    }
}

impl From<ExecutionCost> for SimpleCostSpecification {
    fn from(value: ExecutionCost) -> SimpleCostSpecification {
        let ExecutionCost {
            write_length,
            write_count,
            read_count,
            read_length,
            runtime,
        } = value;
        SimpleCostSpecification {
            write_length: CostFunctions::Constant(write_length),
            write_count: CostFunctions::Constant(write_count),
            read_length: CostFunctions::Constant(read_length),
            read_count: CostFunctions::Constant(read_count),
            runtime: CostFunctions::Constant(runtime),
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_simple_overflows() {
        assert_eq!(
            u64::max_value().cost_overflow_add(1),
            Err(CostErrors::CostOverflow)
        );
        assert_eq!(
            u64::max_value().cost_overflow_mul(2),
            Err(CostErrors::CostOverflow)
        );
        assert_eq!(
            CostFunctions::NLogN(1, 1).compute_cost(u64::max_value()),
            Err(CostErrors::CostOverflow)
        );
    }

    #[test]
    fn test_simple_sub() {
        assert_eq!(0u64.cost_overflow_sub(1), Err(CostErrors::CostOverflow));
    }

    #[test]
    fn test_simple_log2s() {
        let inputs = [
            1,
            2,
            4,
            8,
            16,
            31,
            32,
            33,
            39,
            64,
            128,
            2_u64.pow(63),
            u64::max_value(),
        ];
        let expected = [0, 1, 2, 3, 4, 5, 5, 6, 6, 6, 7, 63, 64];
        for (input, expected) in inputs.iter().zip(expected.iter()) {
            assert_eq!(int_log2(*input).unwrap(), *expected);
        }
    }
}
