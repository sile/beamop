use crate::term::{Atom, Label, List, Literal, Register, Term, XRegister, YRegister};
use crate::{Decode, Encode, Opcode};

#[derive(Debug, Clone, Decode)]
pub enum Op {
    Allocate(AllocateOp),
    AllocateHeap(AllocateHeapOp),
    AllocateHeapZero(AllocateHeapZeroOp),
    Badmatch(BadmatchOp),
    BsGetBinary2(BsGetBinary2Op),
    BsGetInteger2(BsGetInteger2Op),
    BsGetPosition(BsGetPositionOp),
    BsGetTail(BsGetTailOp),
    BsSetPosition(BsSetPositionOp),
    BsStartMatch3(BsStartMatch3Op),
    BsTestTailp(BsTestTail2Op),
    BsTestUnit(BsTestUnitOp),
    BuildStacktrace(BuildStacktraceOp),
    Call(CallOp),
    CallExt(CallExtOp),
    CallExtLast(CallExtLastOp),
    CallExtOnly(CallExtOnlyOp),
    CallOnly(CallOnlyOp),
    Deallocate(DeallocateOp),
    FuncInfo(FuncInfoOp),
    GetList(GetListOp),
    GetTupleElement(GetTupleElementOp),
    InitYregs(InitYregsOp),
    IntCodeEnd(IntCodeEndOp),
    IsEqExact(IsEqExactOp),
    IsNil(IsNilOp),
    IsNonemptyList(IsNonemptyListOp),
    IsTaggedTuple(IsTaggedTupleOp),
    IsTuple(IsTupleOp),
    Jump(JumpOp),
    Label(LabelOp),
    Line(LineOp),
    Move(MoveOp),
    PutList(PutListOp),
    PutTuple2(PutTuple2Op),
    Raise(RaiseOp),
    Return(ReturnOp),
    SelectVal(SelectValOp),
    TestArity(TestArityOp),
    TestHeap(TestHeapOp),
    Try(TryOp),
    TryCase(TryCaseOp),
    TryEnd(TryEndOp),
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(1)]
pub struct LabelOp {
    pub literal: Literal,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(2)]
pub struct FuncInfoOp {
    pub module: Atom,
    pub function: Atom,
    pub arity: Literal,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(3)]
pub struct IntCodeEndOp {}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(4)]
pub struct CallOp {
    pub arity: Literal,
    pub label: Label,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(6)]
pub struct CallOnlyOp {
    pub arity: Literal,
    pub label: Label,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(7)]
pub struct CallExtOp {
    pub arity: Literal,
    pub destination: Literal, // TODO: s/Literal/ImportTableIndex/
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(8)]
pub struct CallExtLastOp {
    pub arity: Literal,
    pub destination: Literal, // TODO: s/Literal/ImportTableIndex/
    pub deallocate: Literal,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(12)]
pub struct AllocateOp {
    pub stack_need: Literal,
    pub live: Literal,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(13)]
pub struct AllocateHeapOp {
    pub stack_need: Literal,
    pub heap_need: Literal,
    pub live: Literal,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(15)]
pub struct AllocateHeapZeroOp {
    pub stack_need: Literal,
    pub heap_need: Literal,
    pub live: Literal,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(16)]
pub struct TestHeapOp {
    pub heap_need: Literal,
    pub live: Literal,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(18)]
pub struct DeallocateOp {
    pub n: Literal,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(19)]
pub struct ReturnOp {}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(43)]
pub struct IsEqExactOp {
    pub label: Label,
    pub arg1: Term,
    pub arg2: Term,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(52)]
pub struct IsNilOp {
    pub label: Label,
    pub arg1: Term,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(56)]
pub struct IsNonemptyListOp {
    pub label: Label,
    pub arg1: Term,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(57)]
pub struct IsTupleOp {
    pub label: Label,
    pub arg1: Term,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(58)]
pub struct TestArityOp {
    pub label: Label,
    pub arg1: Term,
    pub arity: Literal,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(59)]
pub struct SelectValOp {
    pub arg: Term,
    pub fail_label: Label,
    pub destinations: List, // TODO: AssocList
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(61)]
pub struct JumpOp {
    pub label: Label,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(64)]
pub struct MoveOp {
    pub src: Term,
    pub dst: XRegister,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(65)]
pub struct GetListOp {
    pub source: Term,
    pub head: Register,
    pub tail: Register,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(66)]
pub struct GetTupleElementOp {
    pub source: Register,
    pub element: Literal,
    pub destination: Register,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(69)]
pub struct PutListOp {
    pub head: Term,
    pub tail: Term,
    pub destination: Register,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(72)]
pub struct BadmatchOp {
    pub arg1: Term, // TODO
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(78)]
pub struct CallExtOnlyOp {
    pub arity: Literal,
    pub destination: Literal, // TODO: s/Literal/ImportTableIndex/
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(104)]
pub struct TryOp {
    pub register: YRegister,
    pub label: Label,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(105)]
pub struct TryEndOp {
    pub register: YRegister,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(106)]
pub struct TryCaseOp {
    pub register: YRegister,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(108)]
pub struct RaiseOp {
    pub stacktrace: Term,
    pub exc_value: Term,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(117)]
pub struct BsGetInteger2Op {
    pub arg1: Term,
    pub arg2: Term,
    pub arg3: Term,
    pub arg4: Term,
    pub arg5: Term,
    pub arg6: Term,
    pub arg7: Term,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(119)]
pub struct BsGetBinary2Op {
    pub arg1: Term,
    pub arg2: Term,
    pub arg3: Term,
    pub arg4: Term,
    pub arg5: Term,
    pub arg6: Term,
    pub arg7: Term,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(121)]
pub struct BsTestTail2Op {
    pub arg1: Term,
    pub arg2: Term,
    pub arg3: Term,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(131)]
pub struct BsTestUnitOp {
    pub arg1: Term,
    pub arg2: Term,
    pub arg3: Term,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(153)]
pub struct LineOp {
    pub literal: Literal,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(159)]
pub struct IsTaggedTupleOp {
    pub label: Label,
    pub register: XRegister,
    pub arity: Literal,
    pub atom: Atom,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(160)]
pub struct BuildStacktraceOp {}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(164)]
pub struct PutTuple2Op {
    pub destination: Register,
    pub elements: List,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(165)]
pub struct BsGetTailOp {
    pub context: Term,
    pub destination: Register,
    pub live: Literal,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(166)]
pub struct BsStartMatch3Op {
    pub fail: Label,
    pub bin: Term,
    pub live: Literal,
    pub destination: Register,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(167)]
pub struct BsGetPositionOp {
    pub context: Term,
    pub destination: Register,
    pub live: Literal,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(168)]
pub struct BsSetPositionOp {
    pub context: Term,
    pub position: Term,
}

#[derive(Debug, Clone, Opcode, Decode)]
#[opcode(172)]
pub struct InitYregsOp {
    pub registers: Vec<YRegister>,
}
