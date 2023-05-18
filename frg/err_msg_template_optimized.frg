sig ErroneousFlow {
    minimal_subflow: set Src->Sink
}
// "Optimized" version
pred find_erroneous_my_pred_int[ef: ErroneousFlow] {
    some c : Ctrl | {
		(c->ef.minimal_subflow in flow)
    (not predicate[flow])
    (predicate[(flow - (c->ef.minimal_subflow))]) }
}

pred find_erroneous_my_pred {
    some ef1: ErroneousFlow {
        find_erroneous_my_pred_int[ef1]

				no ef2: ErroneousFlow | {
            find_erroneous_my_pred_int[ef2]
            #(ef2.minimal_subflow) < #(ef1.minimal_subflow)
        }
    }
}

test expect {
    create_min_instance: {
        find_erroneous_my_pred
    } for 1 ErroneousFlow for Flows is unsat
}