sig ErroneousFlow {
    minimal_subflow: set Src->Sink
}

// "Minimal" version
pred find_erroneous_my_pred_int[ms: set Src->Sink] {
    some c : Ctrl | {
		(c->ms in flow)
    (not predicate[flow])
    (predicate[(flow - (c->ms))]) }
}

pred find_erroneous_my_pred {
    some ef1: ErroneousFlow {
        find_erroneous_my_pred_int[ef1.minimal_subflow]

        no src: Src, sink: Sink {
						src->sink in ef1.minimal_subflow
            find_erroneous_my_pred_int[ef1.minimal_subflow - src->sink]
        }
    }
}

test expect {
    create_min_instance: {
        find_erroneous_my_pred
    } for 1 ErroneousFlow for Flows is unsat
}