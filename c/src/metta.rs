use hyperon::metta::text::*;
use hyperon::metta::types::AtomType;

use crate::util::*;
use crate::atom::*;
use crate::space::*;

use std::os::raw::*;
use regex::Regex;

// SExprSpace

#[allow(non_camel_case_types)]
pub struct sexpr_space_t {
    space: SExprSpace, 
}

#[no_mangle]
pub extern "C" fn sexpr_space_new() -> *mut sexpr_space_t {
    Box::into_raw(Box::new(sexpr_space_t{ space: SExprSpace::new() })) 
}

#[no_mangle]
pub unsafe extern "C" fn sexpr_space_free(space: *mut sexpr_space_t) {
    drop(Box::from_raw(space)) 
}

// TODO: think how to return the result string in case of error
#[no_mangle]
pub unsafe extern "C" fn sexpr_space_add_str(space: *mut sexpr_space_t, text: *const c_char) -> bool {
    Ok(()) == (*space).space.add_str(cstr_as_str(text))
}

#[allow(non_camel_case_types)]
type atom_constr_t = extern "C" fn(*const c_char, *mut c_void) -> *mut atom_t;

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct droppable_t {
    ptr: *mut c_void,
    free: Option<extern "C" fn(ptr: *mut c_void)>,
}

impl Drop for droppable_t {
    fn drop(&mut self) {
        let free = (*self).free;
        if let Some(free) = free {
            free(self.ptr);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn sexpr_space_register_token(space: *mut sexpr_space_t,
    regex: *const c_char, constr: atom_constr_t, context: droppable_t) {
    let regex = Regex::new(cstr_as_str(regex)).unwrap();
    (*space).space.register_token(regex, move |token| {
        let catom = Box::from_raw(constr(str_as_cstr(token).as_ptr(), context.ptr));
        catom.atom
    });
}

#[no_mangle]
pub unsafe extern "C" fn sexpr_space_into_grounding_space(sexpr: *const sexpr_space_t,
        gnd: *mut grounding_space_t) {
    (*sexpr).space.into_grounding_space(&mut (*gnd).space);
}

#[allow(non_camel_case_types)]
pub struct atom_type_t {
    pub typ: AtomType,
}

#[no_mangle]
pub static ATOM_TYPE_UNDEFINED: &atom_type_t = &atom_type_t{ typ: AtomType::Undefined };

#[no_mangle]
pub unsafe extern "C" fn atom_type_specific(atom: *mut atom_t) -> *mut atom_type_t {
    let c_atom = Box::from_raw(atom);
    Box::into_raw(Box::new(atom_type_t{ typ: AtomType::Specific(c_atom.atom) })) 
}

#[no_mangle]
pub unsafe extern "C" fn atom_type_free(typ: *const atom_type_t) {
    if typ != ATOM_TYPE_UNDEFINED {
        drop(Box::from_raw(typ as *mut atom_type_t)) 
    }
}

#[no_mangle]
pub unsafe extern "C" fn check_type(space: *const grounding_space_t, atom: *const atom_t, typ: *const atom_type_t) -> bool {
    hyperon::metta::types::check_type(&(*space).space, &(*atom).atom, &(*typ).typ)
}

#[no_mangle]
pub unsafe extern "C" fn validate_expr(space: *const grounding_space_t, atom: *const atom_t) -> bool {
    hyperon::metta::types::validate_expr(&(*space).space, &(*atom).atom)
}