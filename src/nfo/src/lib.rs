use std::{collections::HashMap, cell::RefCell, collections::HashSet};
use ic_cdk::export::Principal;
use candid::{Nat, CandidType, Deserialize};

pub type ObjectIdentifier = Nat;

type ErrorTodo = String;

#[derive(CandidType, Deserialize)]
pub enum GenericValue {
    BoolContent(bool),
    TextContent(String),
    BlobContent(Vec<u8>),
    Principal(Principal),
    Nat8Content(u8),
    Nat16Content(u16),
    Nat32Content(u32),
    Nat64Content(u64),
    NatContent(Nat),
    Int8Content(i8),
    Int16Content(i16),
    Int32Content(i32),
    Int64Content(i64),
    // IntContent(Int),
    FloatContent(f64), // motoko only support f64
    NestedContent(Vec<(String, GenericValue)>),
}

#[derive(CandidType, Deserialize)]
pub enum GenericValueSchema {
    BoolContent,
    TextContent,
    BlobContent,
    Principal,
    Nat8Content,
    Nat16Content,
    Nat32Content,
    Nat64Content,
    NatContent,
    Int8Content,
    Int16Content,
    Int32Content,
    Int64Content,
    IntContent,
    FloatContent,
    NestedContent(Vec<(String, GenericValueSchema)>),
}

struct Object {
    owner: Principal,
    fields: HashMap<String, GenericValue>,
}

type ObjectSchema = HashMap<String, GenericValueSchema>;

pub struct Ledger {
    // todo: add whatever more stuff we need
    pub objects: HashMap<ObjectIdentifier, GenericValue>,
}

struct CanisterAccessControlPolicy {
    can_create_new_types: HashSet<Principal>,
    // TODO: add whatever canister-level actions we need
}

struct ObjectAccessControlPolicy {
    can_mint: HashSet<Principal>,
    can_burn: HashSet<Principal>,
    object_schema: ObjectSchema,
    // TODO: figure out how to add the owner to writers of a field
    // TODO: figure if we want fine-grained access control for the nested fields
    field_writers: HashMap<String, HashSet<Principal>>,
} 

type ObjectAccessControlPolicyByType = HashMap<String, ObjectAccessControlPolicy>;

// TODO: add default implementations of the data structures
thread_local!(
    static LEDGER: RefCell<Ledger> = RefCell::new(Ledger::default());
    static CANISTER_ACCESS_CONTROL: RefCell<CanisterAccessControlPolicy> = RefCell::new(CanisterAccessControlPolicy::default());
    static OBJECT_ACCESS_CONTROL: RefCell<ObjectAccessControlPolicyByType>  = RefCell::new(ObjectAccessControlPolicyByType::default());
);


fn check_can_create_objects(caller: Principal, policy: CanisterAccessControlPolicy) -> bool {
    todo!();
}

// TODO: add deserializers to all argument types
// #[ic_cdk_macros::update]
fn new_object_type(object_type_name: String, access_control_policy: ObjectAccessControlPolicy) -> Result<(), ErrorTodo> {
    let caller = ic_cdk::caller();

    // check that the caller can create new object types
    CANISTER_ACCESS_CONTROL.with(|cac| {
        let canister_acp = cac.borrow();
        check_can_create_objects(caller, *canister_acp);
    });

    OBJECT_ACCESS_CONTROL.with(|oac| {
        let object_acp = oac.borrow_mut();
        // check that the object type name is unique for this canister (hasn't been added before)
        assert!(!object_acp.contains_key(&object_type_name));
        object_acp.insert(object_type_name, access_control_policy);
    });
    Ok(())
}

// #[ic_cdk_macros::update]
fn set_value(object_id: ObjectIdentifier, field_name: String, value: GenericValue) -> Result<(), ErrorTodo> {
    let caller = ic_cdk::caller();
    todo!();
    // TODO: check that the value of the field corresponds to the type in the object
    // TODO: check that the caller is allowed to set the field
    // TODO: check that the object exists
    // TODO: change the value of the field
}

// #[ic_cdk_macros::update]
fn mint(new_object_id: Option<ObjectIdentifier>, value: HashMap<String, GenericValue>) -> Result<ObjectIdentifier, ErrorTodo> {
    // TODO: check that the value of the field corresponds to the object schema
    // TODO: check that the caller is allowed to mint
    // TODO: check that the identifier is not already used (if provided)
    // TODO: mint
    todo!()
}

// #[ic_cdk_macros::update]
fn burn(object_id: ObjectIdentifier) -> Result<(), ErrorTodo> {
    let caller = ic_cdk::caller();
    // TODO: check that the object exists
    // TODO: check that the caller is allowed to burn
    // TODO: burn
    todo!()
}