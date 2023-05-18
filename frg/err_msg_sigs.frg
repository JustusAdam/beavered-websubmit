abstract sig Label {}
abstract sig Object {
    labels: set Label
}
sig Function extends Object {}
abstract sig Src extends Object {
    flow: set Sink,
    ctrl_flow: set CallSite,
    types: set Type,
    minimal_subflow: set Sink
}
sig FormalParameter extends Src {
    fp_fun_rel: set Function
}
abstract sig Sink extends Object {}
one sig Return extends Sink {}
sig CallArgument extends Sink {
    arg_call_site: one CallSite
}
sig Type extends Object {
    otype: set Type
}
sig CallSite extends Src {
    function: one Function
}
sig Ctrl extends Function {
    calls: set CallSite
}

