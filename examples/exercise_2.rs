use clap::Parser;
use halo2_base::gates::{RangeChip, RangeInstructions, GateInstructions, range};
use halo2_base::halo2_proofs::plonk::Assigned;
use halo2_base::utils::ScalarField;
use halo2_base::{AssignedValue, QuantumCell};
#[allow(unused_imports)]
use halo2_base::{
    Context,
    QuantumCell::{Constant, Existing, Witness},
};
use halo2_scaffold::scaffold::cmd::Cli;
use halo2_scaffold::scaffold::run;
use serde::{Deserialize, Serialize};
use std::env::var;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub arr: Vec<String>,
    pub end: String,
    pub start: String, // field element, but easier to deserialize as a string
}

// this algorithm takes a public input x, computes x^2 + 72, and outputs the result as public output
fn some_algorithm_in_zk<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,

//     public outputs:
//   * an array `out` of length 1000 such that
//   * the first `end - start` entries of `out` are the subarray `arr[start:end]`
//   * all other entries of `out` are 0.
) {
    let base_arr = input.arr.iter().map(|x| ctx.load_witness(F::from_str_vartime(&x).unwrap())).collect::<Vec<AssignedValue<F>>>();
    make_public.extend(&base_arr);

    // working_arr.push(ctx.load_witness(F::zero()));

    let assigned_elt = ctx.load_witness(F::zero());

    let start = F::from_str_vartime(&input.start).expect("deserialize field element should not fail");
    let end = F::from_str_vartime(&input.end).expect("deserialize field element should not fail");

    let start = ctx.load_witness(start);
    make_public.push(start);

    let end = ctx.load_witness(end);
    make_public.push(end);

    let lookup_bits =
        var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();

    let range_gate: RangeChip<F> = RangeChip::default(lookup_bits);
    let range_bits = 16;

    //pubic params made public

    let start_from_end = range_gate.gate().sub(ctx, Constant(F::from(999)), start);
    let end_from_end = range_gate.gate().sub(ctx, Constant(F::from(999)), end);


    let mut up_working_arr: Vec<AssignedValue<F>> = Vec::new();
    up_working_arr.extend(&base_arr); 

    //set all out of range elements to 0
    let mut working_arr: Vec<AssignedValue<F>> = Vec::new();
    working_arr.push(ctx.load_zero());

    for idx in 0..1000 {
        let cur = up_working_arr.pop().unwrap();
        let cur_idx = Constant(F::from(idx));
        let left_range = range_gate.is_less_than(ctx, cur_idx, end_from_end, range_bits);
        let right_range = range_gate.is_less_than(ctx, start_from_end, cur_idx, range_bits);
        let not_in_selected = range_gate.gate().or(ctx, left_range, right_range); 
        let pushed = range_gate.gate().select(ctx, assigned_elt, cur, not_in_selected);
        working_arr.push(pushed);
    }
    working_arr.push(ctx.load_zero());

    let mut itr_parity = ctx.load_zero();

    // [0, a999, a998, ..., a0, 0], 1002 elts

    // construct a bool vector that gives 1 when elt. is in range and 0 elsewise
    // cannot use range functions on actual values

    for itr in 0..1000{
        itr_parity = range_gate.gate().not(ctx, itr_parity); //starts on 1, removes need for list reversals
        let mut shift_arr: Vec<AssignedValue<F>> = Vec::new(); //vec of length 1000, to hold the running shift array

        let cur_iter = ctx.load_witness(F::from(itr));
        let not_rotated_enough = range_gate.is_less_than(ctx, cur_iter, start, range_bits); //are we done rotating ?

        for _idx in 0..1000 {
            let elt_n = working_arr.pop().unwrap();
            let elt_c = working_arr.pop().unwrap();
            let elt_p = working_arr.pop().unwrap();
            let next_elt_shift = range_gate.gate().select(ctx, elt_p, elt_n, itr_parity);
            let true_next_elt = range_gate.gate().select(ctx, next_elt_shift, elt_c, not_rotated_enough);
            shift_arr.push(true_next_elt);

            working_arr.push(elt_p);
            working_arr.push(elt_c);
        }

        working_arr.pop(); //remove the last "current element", working_arr is just [0]

        if itr < 999 {
            working_arr.extend(&shift_arr); // replacing working_arr
            working_arr.push(ctx.load_zero()); //ending zero
        } else { 
            working_arr = shift_arr; //ends reversed
        }

    };

    working_arr.reverse();
    println!("fin: {:?}", working_arr);

    //vec of only values

    // for idx in 0..1000 {
    // // for _ in 0..20 {
    //     let mut working_arr: Vec<AssignedValue<F>> = Vec::new();
    //     working_arr.push(assigned_elt);
    //     working_arr.extend(&base_arr);

    //     let base_idx = range_gate.gate().add(ctx, Constant(F::from(idx)), start); //find idx in base vec

    //     //boolean
    //     let less_than_end = range_gate.is_less_than(ctx, base_idx, end, range_bits);
    //     let selected_idx = range_gate.gate().select(ctx, base_idx, Constant(F::zero()), less_than_end);

    //     let selected_val = range_gate.gate().select_from_idx(ctx, working_arr, selected_idx);

    //     fin.push(selected_val); //push to final vec
    //     make_public.push(selected_val); //make public

    //     //increment fin_idx
    // };

    // arr has length 1000
    // loop through 1000
    // gets the idx if 

    // let fin_vals = fin.iter().map(|x| x.value()).collect::<Vec<&F>>();

    // println!("Final array: {:?}", fin_vals);

}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    // run different zk commands based on the command line arguments
    run(some_algorithm_in_zk, args);
}