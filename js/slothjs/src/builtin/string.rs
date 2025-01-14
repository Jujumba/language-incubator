use crate::prelude::*;
use crate::{
    object::HostClass, CallContext, Exception, Heap, Interpreted, JSObject, JSRef, JSResult,
};

pub static CLASS: HostClass = HostClass {
    name: "String",
    constructor: string_constructor,
    methods: &[
        ("charAt", string_proto_charAt),
        ("charCodeAt", string_proto_charCodeAt),
        ("indexOf", string_proto_indexOf),
        ("replace", string_proto_replace),
        ("slice", string_proto_slice),
        ("substr", string_proto_substr),
        ("toString", string_proto_valueOf),
        ("valueOf", string_proto_valueOf),
    ],
    static_methods: &[],
};

fn string_constructor(call: CallContext, heap: &mut Heap) -> JSResult<Interpreted> {
    let arg = (call.arguments.first())
        .unwrap_or(&Interpreted::from(""))
        .to_value(heap)?;
    let s = arg.stringify(heap)?;

    if !heap.smells_fresh(call.this_ref) {
        // take the argument and produce a string from it
        return Ok(Interpreted::from(s));
    }

    *heap.get_mut(call.this_ref) = JSObject::from(s);
    Ok(Interpreted::VOID)
}

impl Heap {
    fn ref_to_string(&mut self, href: JSRef) -> JSResult<JSString> {
        match self.get(href).to_primitive() {
            Some(val) => val.stringify(self),
            None => JSValue::from(href).stringify(self),
        }
    }
}

#[allow(non_snake_case)]
fn string_proto_valueOf(call: CallContext, heap: &mut Heap) -> JSResult<Interpreted> {
    let strval = (heap.get(call.this_ref).as_str())
        .ok_or_else(|| Exception::instance_required(call.this_ref, "String"))?;
    Ok(Interpreted::from(strval))
}

#[allow(non_snake_case)]
fn string_proto_charAt(call: CallContext, heap: &mut Heap) -> JSResult<Interpreted> {
    let index = call.arg_as_number(0, heap)?.unwrap_or(0);
    let s = heap.ref_to_string(call.this_ref)?;
    let result = match s.chars().nth(index as usize) {
        Some(c) => c.to_string(),
        None => "".to_string(),
    };
    Ok(Interpreted::from(result))
}

#[allow(non_snake_case)]
fn string_proto_charCodeAt(call: CallContext, heap: &mut Heap) -> JSResult<Interpreted> {
    let index = call.arg_as_number(0, heap)?.unwrap_or(0);
    let s = heap.ref_to_string(call.this_ref)?;
    let result = match s.chars().nth(index as usize) {
        Some(c) => c as i64 as f64,
        None => f64::NAN,
    };
    Ok(Interpreted::from(result))
}

fn string_proto_slice(call: CallContext, heap: &mut Heap) -> JSResult<Interpreted> {
    let s = heap.ref_to_string(call.this_ref)?;
    let strlen = s.chars().count() as i64;

    let begin = match call.arg_as_number(0, heap)?.unwrap_or(0) {
        b if b > strlen => return Ok(Interpreted::from("")),
        b if b <= -strlen => 0,
        b if b < 0 => b + strlen,
        b => b,
    } as usize;
    let end = match call.arg_as_number(1, heap)?.unwrap_or(strlen) {
        e if e <= -strlen => return Ok(Interpreted::from("")),
        e if e < 0 => e + strlen,
        e if e > strlen => strlen,
        e => e,
    } as usize;
    if end < begin {
        return Ok(Interpreted::from(""));
    }

    let substr = &s[begin..end];
    Ok(Interpreted::from(substr))
}

fn string_proto_substr(call: CallContext, heap: &mut Heap) -> JSResult<Interpreted> {
    let s = heap.ref_to_string(call.this_ref)?;
    let strlen = s.chars().count() as i64;
    let begin = match call.arg_as_number(0, heap)?.unwrap_or(0) {
        b if b > strlen => return Ok(Interpreted::from("")),
        b if b < -strlen => 0,
        b if b < 0 => b + strlen,
        b => b,
    } as usize;
    let end = match call.arg_as_number(1, heap)? {
        Some(len) if len <= 0 => begin as i64,
        Some(len) if begin as i64 + len < strlen => begin as i64 + len,
        _ => strlen,
    } as usize;

    let substr = &s[begin..end];
    Ok(Interpreted::from(substr))
}

#[allow(non_snake_case)]
fn string_proto_indexOf(call: CallContext, heap: &mut Heap) -> JSResult<Interpreted> {
    let NOT_FOUND = Interpreted::from(-1);

    let heystack = heap.ref_to_string(call.this_ref)?;
    let strlen = heystack.chars().count() as i64; // COSTLY

    let needle = call.arg_value(0, heap)?.stringify(heap)?;

    let char_start = match call.arg_as_number(1, heap)?.unwrap_or(0) {
        b if b < 0 => 0,
        b if b > strlen && needle.is_empty() => return Ok(Interpreted::from(strlen)),
        b if b > strlen => return Ok(NOT_FOUND),
        b => b,
    };
    // COSTLY
    let byte_start = match heystack.as_str().char_indices().nth(char_start as usize) {
        None => return Ok(NOT_FOUND),
        Some((offset, _)) => offset,
    };
    let heystack = &heystack[byte_start..];

    // COSTLY, but kudos to std::str for providing a fast and tested substring search
    let byte_index = match heystack.find(needle.as_str()) {
        None => return Ok(NOT_FOUND),
        Some(byte_index) => byte_index,
    };

    // COSTLY
    let char_index = (heystack.char_indices())
        .position(|(bi, _)| bi == byte_index)
        .unwrap(); // yes, there must be a position since we've `find`ed it.
    Ok(Interpreted::from(char_start + char_index as i64))
}

fn string_proto_replace(call: CallContext, heap: &mut Heap) -> JSResult<Interpreted> {
    let string = heap.ref_to_string(call.this_ref)?;

    let search = call.arg_value(0, heap)?;
    if let JSValue::Ref(r) = search {
        if r.has_proto(Heap::REGEXP_PROTO, heap) {
            todo!("String.prototype.replace(regexp, ...)");
        }
    }
    let search = search.stringify(heap)?;

    let (before, matched, after) = match string.split_once(search.as_str()) {
        None => return Ok(Interpreted::from(string)),
        Some((before, after)) => (before, &search, after),
    };

    let replace = call.arg_value(1, heap)?;
    if let JSValue::Ref(r) = replace {
        if r.has_proto(Heap::FUNCTION_PROTO, heap) {
            todo!("String.prototype.replace(.. , function)");
        }
    }
    let replace = replace.stringify(heap)?;

    let mut result = String::from(before);
    let mut replace_iter = replace.chars().peekable();
    while let Some(c) = replace_iter.next() {
        if c != '$' {
            result.push(c);
            continue;
        }
        match replace_iter.next_if(|&c| "$`&'".contains(c)) {
            None => result.push(c),
            Some('$') => result.push('$'),
            Some('`') => result.push_str(before),
            Some('&') => result.push_str(matched),
            Some('\'') => result.push_str(after),
            _ => unreachable!(),
        }
    }
    result.push_str(after);
    Ok(Interpreted::from(result))
}
