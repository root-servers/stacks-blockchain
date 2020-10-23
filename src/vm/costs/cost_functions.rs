use super::CostFunctions::{Constant, Linear, LogN, NLogN};
use super::{SimpleCostSpecification, TypeCheckCost};

macro_rules! def_runtime_cost {
    ($Name:ident { $runtime:expr }) => {
        pub const $Name: SimpleCostSpecification = SimpleCostSpecification {
            write_length: Constant(0),
            write_count: Constant(0),
            read_count: Constant(0),
            read_length: Constant(0),
            runtime: $runtime,
        };
    };
}

def_runtime_cost!(ANALYSIS_TYPE_ANNOTATE { Linear(1, 1) });
def_runtime_cost!(ANALYSIS_TYPE_CHECK { Linear(1, 1) });
def_runtime_cost!(ANALYSIS_TYPE_LOOKUP { Linear(1, 1) });
def_runtime_cost!(ANALYSIS_VISIT { Constant(1) });
def_runtime_cost!(ANALYSIS_ITERABLE_FUNC { Constant(1) });
def_runtime_cost!(ANALYSIS_OPTION_CONS { Constant(1) });
def_runtime_cost!(ANALYSIS_OPTION_CHECK { Constant(1) });
def_runtime_cost!(ANALYSIS_BIND_NAME { Linear(1, 1) });
def_runtime_cost!(ANALYSIS_LIST_ITEMS_CHECK { Linear(1, 1) });
def_runtime_cost!(ANALYSIS_CHECK_TUPLE_GET { LogN(1, 1) });
def_runtime_cost!(ANALYSIS_CHECK_TUPLE_CONS { NLogN(1, 1) });
def_runtime_cost!(ANALYSIS_TUPLE_ITEMS_CHECK { Linear(1, 1) });
def_runtime_cost!(ANALYSIS_CHECK_LET { Linear(1, 1) });

def_runtime_cost!(ANALYSIS_LOOKUP_FUNCTION { Constant(1) });
def_runtime_cost!(ANALYSIS_LOOKUP_FUNCTION_TYPES { Linear(1, 1) });

def_runtime_cost!(ANALYSIS_LOOKUP_VARIABLE_CONST { Constant(1) });
def_runtime_cost!(ANALYSIS_LOOKUP_VARIABLE_DEPTH { NLogN(1, 1) });

def_runtime_cost!(AST_PARSE { Linear(1, 1) });
def_runtime_cost!(AST_CYCLE_DETECTION { Linear(1, 1) });

pub const ANALYSIS_STORAGE: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Linear(1, 1),
    write_count: Constant(1),
    runtime: Linear(1, 1),
    read_count: Constant(1),
    read_length: Constant(1),
};

pub const ANALYSIS_USE_TRAIT_ENTRY: SimpleCostSpecification = SimpleCostSpecification {
    // increases the total storage consumed by the contract!
    //  so we count the additional write_length, but since it does _not_ require
    //  an additional _write_, we don't charge for that.
    write_length: Linear(1, 1),
    write_count: Constant(0),
    runtime: Linear(1, 1),
    read_count: Constant(1),
    read_length: Linear(1, 1),
};

pub const ANALYSIS_GET_FUNCTION_ENTRY: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Constant(0),
    write_count: Constant(0),
    runtime: Linear(1, 1),
    read_count: Constant(1),
    read_length: Linear(1, 1),
};

pub const ANALYSIS_FETCH_CONTRACT_ENTRY: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Constant(0),
    write_count: Constant(0),
    runtime: Linear(1, 1),
    read_count: Constant(1),
    read_length: Linear(1, 1),
};

def_runtime_cost!(LOOKUP_VARIABLE_DEPTH { Linear(1, 1) });
def_runtime_cost!(LOOKUP_VARIABLE_SIZE { Linear(1, 0) });
def_runtime_cost!(LOOKUP_FUNCTION { Constant(1) });
def_runtime_cost!(BIND_NAME { Constant(1) });
def_runtime_cost!(INNER_TYPE_CHECK_COST { Linear(1, 1) });
def_runtime_cost!(USER_FUNCTION_APPLICATION { Linear(1, 1) });

def_runtime_cost!(LET { Linear(1, 1) });
def_runtime_cost!(IF { Constant(1) });
def_runtime_cost!(ASSERTS { Constant(1) });
def_runtime_cost!(MAP { Constant(1) });
def_runtime_cost!(FILTER { Constant(1) });
def_runtime_cost!(LEN { Constant(1) });
def_runtime_cost!(FOLD { Constant(1) });
def_runtime_cost!(LIST_CONS { Linear(1, 1) });
def_runtime_cost!(TYPE_PARSE_STEP { Constant(1) });
def_runtime_cost!(DATA_HASH_COST { Linear(1, 1) });
def_runtime_cost!(TUPLE_GET { NLogN(1, 1) });
def_runtime_cost!(TUPLE_CONS { NLogN(1, 1) });

def_runtime_cost!(ADD { Linear(1, 1) });
def_runtime_cost!(SUB { Linear(1, 1) });
def_runtime_cost!(MUL { Linear(1, 1) });
def_runtime_cost!(DIV { Linear(1, 1) });
def_runtime_cost!(GEQ { Constant(1) });
def_runtime_cost!(LEQ { Constant(1) });
def_runtime_cost!(LE  { Constant(1) });
def_runtime_cost!(GE  { Constant(1) });
def_runtime_cost!(INT_CAST { Constant(1) });
def_runtime_cost!(MOD { Constant(1) });
def_runtime_cost!(POW { Constant(1) });
def_runtime_cost!(SQRTI { Constant(1) });
def_runtime_cost!(XOR { Constant(1) });
def_runtime_cost!(NOT { Constant(1) });
def_runtime_cost!(EQ { Linear(1, 1) });
def_runtime_cost!(BEGIN { Constant(1) });
def_runtime_cost!(HASH160 { Constant(1) });
def_runtime_cost!(SHA256 { Constant(1) });
def_runtime_cost!(SHA512 { Constant(1) });
def_runtime_cost!(SHA512T256 { Constant(1) });
def_runtime_cost!(KECCAK256 { Constant(1) });
def_runtime_cost!(SECP256K1RECOVER { Constant(1) });
def_runtime_cost!(SECP256K1VERIFY { Constant(1) });
def_runtime_cost!(PRINT { Linear(1, 1) });
def_runtime_cost!(SOME_CONS { Constant(1) });
def_runtime_cost!(OK_CONS { Constant(1) });
def_runtime_cost!(ERR_CONS { Constant(1) });
def_runtime_cost!(DEFAULT_TO { Constant(1) });
def_runtime_cost!(UNWRAP_RET { Constant(1) });
def_runtime_cost!(UNWRAP_ERR_OR_RET { Constant(1) });
def_runtime_cost!(IS_OKAY { Constant(1) });
def_runtime_cost!(IS_NONE { Constant(1) });
def_runtime_cost!(IS_ERR { Constant(1) });
def_runtime_cost!(IS_SOME { Constant(1) });
def_runtime_cost!(UNWRAP { Constant(1) });
def_runtime_cost!(UNWRAP_ERR { Constant(1) });
def_runtime_cost!(TRY_RET { Constant(1) });
def_runtime_cost!(MATCH { Constant(1) });
def_runtime_cost!(OR { Linear(1, 1) });
def_runtime_cost!(AND { Linear(1, 1) });

def_runtime_cost!(APPEND { Linear(1, 1) });
def_runtime_cost!(CONCAT { Linear(1, 1) });
def_runtime_cost!(AS_MAX_LEN { Constant(1) });

def_runtime_cost!(CONTRACT_CALL { Constant(1) });
def_runtime_cost!(CONTRACT_OF { Constant(1) });
def_runtime_cost!(PRINCIPAL_OF { Constant(1) });

pub const AT_BLOCK: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Constant(0),
    write_count: Constant(0),
    runtime: Constant(1),
    read_count: Constant(1),
    read_length: Constant(1),
};

pub const LOAD_CONTRACT: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Constant(0),
    write_count: Constant(0),
    runtime: Linear(1, 1),
    read_count: Constant(1),
    read_length: Linear(1, 1),
};

pub const CREATE_MAP: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Linear(1, 1),
    write_count: Constant(1),
    runtime: Linear(1, 1),
    read_count: Constant(0),
    read_length: Constant(0),
};

pub const CREATE_VAR: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Linear(1, 1),
    write_count: Constant(2),
    runtime: Linear(1, 1),
    read_count: Constant(0),
    read_length: Constant(0),
};

pub const CREATE_NFT: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Linear(1, 1),
    write_count: Constant(1),
    runtime: Linear(1, 1),
    read_count: Constant(0),
    read_length: Constant(0),
};

pub const CREATE_FT: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Constant(1),
    write_count: Constant(2),
    runtime: Constant(1),
    read_count: Constant(0),
    read_length: Constant(0),
};

pub const FETCH_ENTRY: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Constant(0),
    write_count: Constant(0),
    runtime: Linear(1, 1),
    read_count: Constant(1),
    read_length: Linear(1, 1),
};

pub const SET_ENTRY: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Linear(1, 1),
    write_count: Constant(1),
    runtime: Linear(1, 1),
    read_count: Constant(1),
    read_length: Constant(0),
};

pub const FETCH_VAR: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Constant(0),
    write_count: Constant(0),
    runtime: Linear(1, 1),
    read_count: Constant(1),
    read_length: Linear(1, 1),
};

pub const SET_VAR: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Linear(1, 1),
    write_count: Constant(1),
    runtime: Linear(1, 1),
    read_count: Constant(1),
    read_length: Constant(0),
};

pub const CONTRACT_STORAGE: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Linear(1, 1),
    write_count: Constant(1),
    runtime: Linear(1, 1),
    read_count: Constant(0),
    read_length: Constant(0),
};

pub const BLOCK_INFO: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Constant(0),
    write_count: Constant(0),
    runtime: Constant(1),
    read_count: Constant(1),
    read_length: Constant(1),
};

pub const STX_BALANCE: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Constant(0),
    write_count: Constant(0),
    runtime: Constant(1),
    read_count: Constant(1),
    read_length: Constant(1),
};

pub const STX_TRANSFER: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Constant(1),
    write_count: Constant(1),
    runtime: Constant(1),
    read_count: Constant(1),
    read_length: Constant(1),
};

pub const FT_MINT: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Constant(1),
    write_count: Constant(2),
    runtime: Constant(1),
    read_count: Constant(2),
    read_length: Constant(1),
};

pub const FT_TRANSFER: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Constant(1),
    write_count: Constant(2),
    runtime: Constant(1),
    read_count: Constant(2),
    read_length: Constant(1),
};

pub const FT_BALANCE: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Constant(0),
    write_count: Constant(0),
    runtime: Constant(1),
    read_count: Constant(1),
    read_length: Constant(1),
};

pub const NFT_MINT: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Constant(1),
    write_count: Constant(1),
    runtime: Linear(1, 1),
    read_count: Constant(1),
    read_length: Constant(1),
};

pub const NFT_TRANSFER: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Constant(1),
    write_count: Constant(1),
    runtime: Linear(1, 1),
    read_count: Constant(1),
    read_length: Constant(1),
};

pub const NFT_OWNER: SimpleCostSpecification = SimpleCostSpecification {
    write_length: Constant(0),
    write_count: Constant(0),
    runtime: Linear(1, 1),
    read_count: Constant(1),
    read_length: Constant(1),
};

pub const TYPE_CHECK_COST: TypeCheckCost = TypeCheckCost {};

define_named_enum!(ClarityCostFunction {
    AnalysisTypeAnnotate("cost_analysis_type_annotate"),
    AnalysisTypeCheck("cost_analysis_type_check"),
    AnalysisTypeLookup("cost_analysis_type_lookup"),
    AnalysisVisit("cost_analysis_visit"),
    AnalysisIterableFunc("cost_analysis_iterable_func"),
    AnalysisOptionCons("cost_analysis_option_cons"),
    AnalysisOptionCheck("cost_analysis_option_check"),
    AnalysisBindName("cost_analysis_bind_name"),
    AnalysisListItemsCheck("cost_analysis_list_items_check"),
    AnalysisCheckTupleGet("cost_analysis_check_tuple_get"),
    AnalysisCheckTupleCons("cost_analysis_check_tuple_cons"),
    AnalysisTupleItemsCheck("cost_analysis_tuple_items_check"),
    AnalysisCheckLet("cost_analysis_check_let"),
    AnalysisLookupFunction("cost_analysis_lookup_function"),
    AnalysisLookupFunctionTypes("cost_analysis_lookup_function_types"),
    AnalysisLookupVariableConst("cost_analysis_lookup_variable_const"),
    AnalysisLookupVariableDepth("cost_analysis_lookup_variable_depth"),
    AstParse("cost_ast_parse"),
    AstCycleDetection("cost_ast_cycle_detection"),
    AnalysisStorage("cost_analysis_storage"),
    AnalysisUseTraitEntry("cost_analysis_use_trait_entry"),
    AnalysisGetFunctionEntry("cost_analysis_get_function_entry"),
    AnalysisFetchContractEntry("cost_analysis_fetch_contract_entry"),
    LookupVariableDepth("cost_lookup_variable_depth"),
    LookupVariableSize("cost_lookup_variable_size"),
    LookupFunction("cost_lookup_function"),
    BindName("cost_bind_name"),
    InnerTypeCheckCost("cost_inner_type_check_cost"),
    UserFunctionApplication("cost_user_function_application"),
    Let("cost_let"),
    If("cost_if"),
    Asserts("cost_asserts"),
    Map("cost_map"),
    Filter("cost_filter"),
    Len("cost_len"),
    Fold("cost_fold"),
    ListCons("cost_list_cons"),
    TypeParseStep("cost_type_parse_step"),
    DataHashCost("cost_data_hash_cost"),
    TupleGet("cost_tuple_get"),
    TupleCons("cost_tuple_cons"),
    Add("cost_add"),
    Sub("cost_sub"),
    Mul("cost_mul"),
    Div("cost_div"),
    Geq("cost_geq"),
    Leq("cost_leq"),
    Le("cost_le"),
    Ge("cost_ge"),
    IntCast("cost_int_cast"),
    Mod("cost_mod"),
    Pow("cost_pow"),
    Sqrti("cost_sqrti"),
    Xor("cost_xor"),
    Not("cost_not"),
    Eq("cost_eq"),
    Begin("cost_begin"),
    Hash160("cost_hash160"),
    Sha256("cost_sha256"),
    Sha512("cost_sha512"),
    Sha512t256("cost_sha512t256"),
    Keccak256("cost_keccak256"),
    Secp256k1recover("cost_secp256k1recover"),
    Secp256k1verify("cost_secp256k1verify"),
    Print("cost_print"),
    SomeCons("cost_some_cons"),
    OkCons("cost_ok_cons"),
    ErrCons("cost_err_cons"),
    DefaultTo("cost_default_to"),
    UnwrapRet("cost_unwrap_ret"),
    UnwrapErrOrRet("cost_unwrap_err_or_ret"),
    IsOkay("cost_is_okay"),
    IsNone("cost_is_none"),
    IsErr("cost_is_err"),
    IsSome("cost_is_some"),
    Unwrap("cost_unwrap"),
    UnwrapErr("cost_unwrap_err"),
    TryRet("cost_try_ret"),
    Match("cost_match"),
    Or("cost_or"),
    And("cost_and"),
    Append("cost_append"),
    Concat("cost_concat"),
    AsMaxLen("cost_as_max_len"),
    ContractCall("cost_contract_call"),
    ContractOf("cost_contract_of"),
    PrincipalOf("cost_principal_of"),
    AtBlock("cost_at_block"),
    LoadContract("cost_load_contract"),
    CreateMap("cost_create_map"),
    CreateVar("cost_create_var"),
    CreateNft("cost_create_nft"),
    CreateFt("cost_create_ft"),
    FetchEntry("cost_fetch_entry"),
    SetEntry("cost_set_entry"),
    FetchVar("cost_fetch_var"),
    SetVar("cost_set_var"),
    ContractStorage("cost_contract_storage"),
    BlockInfo("cost_block_info"),
    StxBalance("cost_stx_balance"),
    StxTransfer("cost_stx_transfer"),
    FtMint("cost_ft_mint"),
    FtTransfer("cost_ft_transfer"),
    FtBalance("cost_ft_balance"),
    NftMint("cost_nft_mint"),
    NftTransfer("cost_nft_transfer"),
    NftOwner("cost_nft_owner"),
});
