/*
 * Open questions:
 *  - how do I specify that only the owner can, e.g., burn an object
 *  - 
 */
use std::{collections::HashMap, cell::RefCell, collections::HashSet};
use ic_cdk::export::Principal;
use candid::{Nat, CandidType, Deserialize};
// use itertools::Itertools;
extern crate derive_more;
// use the derives that you want in the file
use derive_more::Display;

pub type ObjectId = Nat;
pub type ObjectType = String;
pub type FieldName = String;

#[derive(Debug)]
pub enum NFOError {
    NoSuchObjectError { object_id: ObjectId },
    NoSuchFieldError { field_name: FieldName },
    SchemaMismatchError,
    NotAuthorizedError,
    DuplicateObjectIdError { object_id: ObjectId },
    DuplicateObjectTypeError { object_type: ObjectType },
}

use NFOError::*;

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

pub struct Object {
    owner: Principal,
    object_type: ObjectType,
    fields: HashMap<FieldName, GenericValue>,
}

type ObjectSchema = HashMap<FieldName, GenericValueSchema>;

#[derive(Default)]
pub struct Ledger {
    // todo: add whatever more stuff we need
    pub objects: HashMap<ObjectId, Object>,
}

#[derive(Default)]
pub struct CanisterAccessControlPolicy {
    can_create_new_types: HashSet<Principal>,
    // TODO: add whatever canister-level actions we need
}

#[derive(PartialEq, Eq, Hash, Debug, Display)]
pub enum Actor {
    Owner,
    Fixed(Principal),
}

pub struct ObjectAccessControlPolicy {
    can_mint: HashSet<Principal>,
    can_burn: HashSet<Actor>,
    object_schema: ObjectSchema,
    // TODO: figure out how to add the owner to writers of a field
    // TODO: figure if we want fine-grained access control for the nested fields
    field_writers: HashMap<FieldName, HashSet<Actor>>,
} 

type ObjectAccessControlPolicyByType = HashMap<ObjectType, ObjectAccessControlPolicy>;

fn check_value_matches_value_schema(value: &GenericValue, schema: &GenericValueSchema) -> Result<(), NFOError> {
    match (value, schema) {
        (GenericValue::BoolContent(_), GenericValueSchema::BoolContent) => Ok(()),
        (GenericValue::BlobContent(_), GenericValueSchema::BlobContent) => Ok(()),
        (GenericValue::FloatContent(_), GenericValueSchema::FloatContent) => Ok(()),
        (GenericValue::Int8Content(_), GenericValueSchema::Int8Content) => Ok(()),
        (GenericValue::Int16Content(_), GenericValueSchema::Int16Content) => Ok(()),
        (GenericValue::Int32Content(_), GenericValueSchema::Int32Content) => Ok(()),
        (GenericValue::Int64Content(_), GenericValueSchema::Int64Content) => Ok(()),
        (GenericValue::Nat8Content(_), GenericValueSchema::Nat8Content) => Ok(()),
        (GenericValue::Nat16Content(_), GenericValueSchema::Nat16Content) => Ok(()),
        (GenericValue::Nat32Content(_), GenericValueSchema::Nat32Content) => Ok(()),
        (GenericValue::Nat64Content(_), GenericValueSchema::Nat64Content) => Ok(()),
        (GenericValue::TextContent(_), GenericValueSchema::TextContent) => Ok(()),
        (GenericValue::Principal(_), GenericValueSchema::Principal) => Ok(()),
        (GenericValue::NestedContent(cnt), GenericValueSchema::NestedContent(cnt_schema)) if cnt.len() == cnt_schema.len() => {
            for (f, f_schema) in cnt.iter().zip(cnt_schema) {
                if f.0 != f_schema.0 { return Err(SchemaMismatchError) };
                let _ = check_value_matches_value_schema(&f.1, &f_schema.1)?;
            }
            Ok(())
        }
        _ => Err(SchemaMismatchError),
    }
}

fn check_value_matches_field_schema(schema: &ObjectSchema, field_name: &FieldName, value: &GenericValue) -> Result<(), NFOError> {
    schema.get(field_name).map_or(
        Err(NoSuchFieldError { field_name: field_name.clone() }), 
        |value_schema| check_value_matches_value_schema(value, value_schema))
}

fn check_caller_allowed(caller: &Principal, owner: &Principal, actors: &HashSet<Actor>) -> Result<(), NFOError> {
    if actors.contains(&Actor::Fixed(caller.clone())) || (caller == owner && actors.contains(&Actor::Owner)) {
        Ok(())
    } else { 
        Err(NotAuthorizedError)
    }
}

pub fn set_value_impl(
    caller: &Principal, 
    ledger: &mut Ledger, 
    oac_by_type: &ObjectAccessControlPolicyByType, 
    object_id: ObjectId, 
    field_name: FieldName, 
    value: GenericValue) -> Result<(), NFOError>  {
        let obj = ledger.objects.get_mut(&object_id).ok_or(NoSuchObjectError { object_id: object_id })?;
        let oac = oac_by_type.get(&obj.object_type).unwrap();
        let schema = &oac.object_schema;
        check_value_matches_field_schema(&schema, &field_name, &value)?;
        let field_writers = &oac.field_writers.get(&field_name).ok_or(NoSuchFieldError { field_name: field_name.clone() })?;
        check_caller_allowed(caller, &obj.owner, field_writers)?;
        obj.fields.insert(field_name, value);
        Ok(())

}

fn format_policy(field_writers: &HashMap<FieldName, HashSet<Actor>>) -> String {
    field_writers.iter().map(|(k, v)| 
        format!("{}: {:#?}\n", k, v)).collect()
}

pub fn display_policy_impl(ledger: &Ledger, oac_by_type: &ObjectAccessControlPolicyByType, object_id: ObjectId) -> Result<String, NFOError> {
    let obj = ledger.objects.get(&object_id).ok_or(NoSuchObjectError { object_id: object_id })?;
    let oac = oac_by_type.get(&obj.object_type).unwrap();
    let field_writers = &oac.field_writers;
    Ok(format_policy(field_writers))
}



fn allocate_fresh_id<V>(objects: &HashMap<ObjectId, V>) -> ObjectId {
    // TODO: Not exactly efficient
    for i in 1..objects.len() {
        if !objects.contains_key(&Nat::from(i)) {
            return Nat::from(i);
        }
    }
    Nat::from(objects.len() + 1)
}

fn check_value_matches_object_schema(value: &HashMap<FieldName, GenericValue>, schema: &ObjectSchema) -> Result<(), NFOError>{
    if value.len() != schema.len() { return Err(SchemaMismatchError) };
    for (name, value) in value.iter() {
        let value_schema = schema.get(name).ok_or(SchemaMismatchError)?;
        check_value_matches_value_schema(value, value_schema)?;
    }
    Ok(())
}

pub fn mint_impl(caller: &Principal, ledger: &mut Ledger, oac_by_type: &ObjectAccessControlPolicyByType, new_object_id: Option<ObjectId>, object_type: ObjectType, owner: Principal, value: HashMap<FieldName, GenericValue>) -> Result<ObjectId, NFOError> {
    let object_id = match new_object_id {
        None => Ok(allocate_fresh_id(&ledger.objects)),
        Some(id) if ledger.objects.contains_key(&id) => Err(DuplicateObjectIdError { object_id: id }),
        Some(id) => Ok(id),
    }?;
    let oac = oac_by_type.get(&object_type).unwrap();
    let schema = &oac.object_schema;
    let _ = if oac.can_mint.contains(&caller) { Ok(()) } else { Err(NotAuthorizedError) }?;
    check_value_matches_object_schema(&value, &schema)?;
    ledger.objects.insert(object_id.clone(), Object { owner: owner, object_type: object_type, fields: value });
    Ok(object_id)
}


pub fn burn_impl(caller: &Principal, ledger: &mut Ledger, oac_by_type: &ObjectAccessControlPolicyByType, object_id: ObjectId) -> Result<(), NFOError> {
    let obj = ledger.objects.get_mut(&object_id).ok_or(NoSuchObjectError { object_id: object_id.clone() })?;
    let oac = oac_by_type.get(&obj.object_type).unwrap();
    let _ = check_caller_allowed(&caller, &obj.owner, &oac.can_burn)?;
    ledger.objects.remove(&object_id);
    Ok(())

}


pub fn add_object_type_impl(
    caller: &Principal, 
    cac: &CanisterAccessControlPolicy, 
    oac_by_type: &mut ObjectAccessControlPolicyByType, 
    object_type: ObjectType, 
    policy: ObjectAccessControlPolicy) -> Result<(), NFOError> {

    if !oac_by_type.contains_key(&object_type) { Ok(()) } else { Err(DuplicateObjectTypeError { object_type: object_type.clone() })}?;
    if cac.can_create_new_types.contains(&caller) { Ok(()) } else { Err(NotAuthorizedError )}?;
    oac_by_type.insert(object_type, policy);
    Ok(())

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_workflow() {
        let _ledger = Ledger::default();
        // let admin = Principal::from("p1");
        let _cac = CanisterAccessControlPolicy::default();
        let _oac_by_type = ObjectAccessControlPolicyByType::default();
    }
}

thread_local!(
    static LEDGER: RefCell<Ledger> = RefCell::new(Ledger::default());
    static CANISTER_ACCESS_CONTROL: RefCell<CanisterAccessControlPolicy> = RefCell::new(CanisterAccessControlPolicy::default());
    static OBJECT_ACCESS_CONTROL: RefCell<ObjectAccessControlPolicyByType>  = RefCell::new(ObjectAccessControlPolicyByType::default());
);

pub fn add_object_type(object_type: ObjectType, policy: ObjectAccessControlPolicy) -> Result<(), NFOError> {
    let caller = ic_cdk::caller();
    CANISTER_ACCESS_CONTROL.with(|c| { OBJECT_ACCESS_CONTROL.with(|o| {
        let cac = c.borrow_mut();
        let mut oac_by_type = o.borrow_mut();
        add_object_type_impl(&caller, &cac, &mut oac_by_type, object_type, policy)
    })})
}

// #[ic_cdk_macros::update]
pub fn burn(object_id: ObjectId) -> Result<(), NFOError> {
    let caller = ic_cdk::caller();
    LEDGER.with(|l| { OBJECT_ACCESS_CONTROL.with(|o| {
        let mut ledger = l.borrow_mut();
        let o_borrow = o.borrow();
        burn_impl(&caller, &mut ledger, &o_borrow, object_id)
    })})
}

// #[ic_cdk_macros::update]
pub fn set_value(object_id: ObjectId, field_name: FieldName, value: GenericValue) -> Result<(), NFOError> {
    let caller = ic_cdk::caller();

    LEDGER.with(|l| { OBJECT_ACCESS_CONTROL.with(|o| {
        let mut ledger = l.borrow_mut();
        let o_borrow = o.borrow();
        set_value_impl(&caller, &mut ledger, &o_borrow, object_id, field_name, value)
    })})
}

pub fn display_policy(object_id: ObjectId) -> Result<String, NFOError> {
    LEDGER.with(|l| { OBJECT_ACCESS_CONTROL.with(|o| {
        let ledger = l.borrow();
        let o_borrow = o.borrow();
        display_policy_impl(&ledger, &o_borrow, object_id)
    })})
}

// #[ic_cdk_macros::update]
pub fn mint(new_object_id: Option<ObjectId>, object_type: ObjectType, owner: Principal, value: HashMap<FieldName, GenericValue>) -> Result<ObjectId, NFOError> {
    let caller = ic_cdk::caller();
    LEDGER.with(|l| { OBJECT_ACCESS_CONTROL.with(|o| {
        let mut ledger = l.borrow_mut();
        let o_borrow = o.borrow();
        mint_impl(&caller, &mut ledger, &o_borrow, new_object_id, object_type, owner, value)
    })})
}

