use crate::{
    ast::Identifier,
    runtime::{
        self,
        EvalResult,
    },
    CallContext,
    Exception,
    Heap,
    HeapNode,
    Interpretable,
    Interpreted,
    JSRef,
    Program,
};

pub struct EsprimaParser {
    object: JSRef,
    esparse: JSRef,
}

impl EsprimaParser {
    const ESPRIMA: &'static str = include_str!("../../tmp/esprima.json");
}

impl runtime::Parser for EsprimaParser {
    fn load(heap: &mut Heap) -> EvalResult<Self> {
        let esprima_json = serde_json::from_str::<serde_json::Value>(Self::ESPRIMA)?;

        let esprima = Program::parse_from(&esprima_json).map_err(Exception::SyntaxTreeError)?;

        esprima.interpret(heap)?;

        let object: JSRef = heap.lookup_path(&["esprima"])?.to_ref(heap)?;
        let esparse = (heap.get(object))
            .get_value("parse")
            .ok_or_else(|| Exception::ReferenceNotFound(Identifier::from("esprima.parse")))?
            .to_ref()?;
        Ok(EsprimaParser { object, esparse })
    }

    fn parse(&self, input: &str, heap: &mut Heap) -> EvalResult<Program> {
        let estree: Interpreted = heap.execute(
            self.esparse,
            CallContext {
                this_ref: self.object,
                method_name: "parse".to_string(),
                arguments: vec![Interpreted::from(input) /*{ loc: true },*/],
                loc: None,
            },
        )?;
        let node = estree.to_ref(heap)?;

        let program = HeapNode::with(heap, node, |estree| Program::parse_from(estree))
            .map_err(Exception::invalid_ast)?;
        Ok(program)
    }
}