use clap::Parser;
use halo2_base::gates::{RangeChip, RangeInstructions, GateInstructions};
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

    // let mut working_arr: Vec<AssignedValue<F>> = Vec::new();
    // working_arr.extend(&base_arr);
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

    let mut fin: Vec<AssignedValue<F>> = Vec::new();
    let mut fin_idx = ctx.load_witness(F::zero());

    for _ in 0..1000 {
    // for _ in 0..20 {
        let mut working_arr: Vec<AssignedValue<F>> = Vec::new();
        working_arr.push(assigned_elt);
        working_arr.extend(&base_arr);

        let base_idx = range_gate.gate().add(ctx, fin_idx, start); //find idx in base vec

        //boolean
        let less_than_end = range_gate.is_less_than(ctx, base_idx, end, range_bits);
        let selected_idx = range_gate.gate().select(ctx, base_idx, Constant(F::zero()), less_than_end);

        let selected_val = range_gate.gate().select_from_idx(ctx, working_arr, selected_idx);

        fin.push(selected_val); //push to final vec
        make_public.push(selected_val); //make public

        //increment fin_idx
        fin_idx = range_gate.gate().add(ctx, fin_idx, Constant(F::one()));
    };

    // arr has length 1000
    // loop through 1000
    // gets the idx if 

    let fin_vals = fin.iter().map(|x| x.value()).collect::<Vec<&F>>();

    println!("Final array: {:?}", fin_vals);

}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    // run different zk commands based on the command line arguments
    run(some_algorithm_in_zk, args);
}