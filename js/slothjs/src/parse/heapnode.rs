use crate::prelude::*;
use crate::source;
use crate::Heap;

use super::*;

/// `HeapNode` contains a heap reference and a `JSRef` to an AST subtree in it.
/// It implements [`SourceNode`] for on-heap AST trees.
#[derive(Clone)]
pub struct HeapNode {
    heap: Rc<Heap>,
    node: JSRef,
}

impl HeapNode {
    pub fn with<T, F>(heap: &mut Heap, node: JSRef, mut action: F) -> T
    where
        F: FnMut(&HeapNode) -> T,
    {
        let mut tmp = Heap::new();
        core::mem::swap(heap, &mut tmp);
        let heapptr = Rc::new(tmp);
        let heapnode = HeapNode {
            heap: heapptr,
            node,
        };

        let result = action(&heapnode);

        tmp = Rc::try_unwrap(heapnode.heap).expect("only one reference left");
        core::mem::swap(heap, &mut tmp);
        result
    }

    pub fn with_node(&self, node: JSRef) -> Self {
        HeapNode {
            heap: Rc::clone(&self.heap),
            node,
        }
    }
}

impl fmt::Debug for HeapNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HeapNode{{ {:?} }}", self.node)
    }
}

impl SourceNode for HeapNode {
    type Error = Self;

    fn to_error(&self) -> Self::Error {
        self.clone()
    }

    fn get_location(&self) -> Option<source::Location> {
        None // TODO
    }

    fn get_literal(&self, property: &str) -> ParseResult<Literal, Self::Error> {
        let child = (self.heap.get(self.node))
            .get_value(property)
            .ok_or_else(|| ParseError::no_attr(property, self.to_error()))?;
        let json = child
            .to_json(&self.heap)
            .map_err(|e| ParseError::InvalidJSON {
                err: format!("{:?}", e),
            })?;
        Ok(Literal(json))
    }

    fn get_bool(&self, property: &str) -> ParseResult<bool, Self::Error> {
        let value = (self.heap.get(self.node))
            .get_value(property)
            .ok_or_else(|| ParseError::no_attr(property, self.to_error()))?;
        match value {
            JSValue::Bool(b) => Ok(b),
            _ => Err(ParseError::ShouldBeBool {
                value: self.to_error(),
            }),
        }
    }

    fn get_str(&self, property: &str) -> ParseResult<String, Self::Error> {
        let value = (self.heap.get(self.node))
            .get_value(property)
            .ok_or_else(|| ParseError::no_attr(property, self.to_error()))?;
        match value {
            JSValue::String(s) => Ok(s),
            _ => Err(ParseError::ShouldBeString {
                value: self.to_error(),
            }),
        }
    }

    fn map_node<T, F>(&self, property: &str, mut action: F) -> ParseResult<T, Self::Error>
    where
        F: FnMut(&Self) -> ParseResult<T, Self::Error>,
    {
        let node = self.heap.get(self.node);
        match node.get_value(property) {
            Some(JSValue::Ref(childref)) => {
                let child = self.with_node(childref);
                Ok(action(&child)?)
            }
            _ => Err(ParseError::no_attr(property, self.to_error())),
        }
    }

    fn map_array<T, F>(&self, property: &str, mut func: F) -> ParseResult<Vec<T>, Self::Error>
    where
        F: FnMut(&Self) -> ParseResult<T, Self::Error>,
    {
        let value = (self.heap.get(self.node))
            .get_value(property)
            .ok_or_else(|| ParseError::no_attr(property, self.to_error()))?;
        let arrref = value.to_ref().map_err(|_| ParseError::ShouldBeArray {
            value: self.to_error(),
        })?;
        let array =
            (self.heap.get(arrref))
                .as_array()
                .ok_or_else(|| ParseError::ShouldBeArray {
                    value: self.to_error(),
                })?;
        array
            .storage
            .iter()
            .map(|child| {
                let childref = child.to_ref().map_err(|_| ParseError::UnexpectedValue {
                    want: "objects",
                    value: self.to_error(),
                })?;
                let child = self.with_node(childref);
                func(&child)
            })
            .collect()
    }
}
