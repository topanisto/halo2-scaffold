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
    let end_idx = range_gate.gate().sub(ctx, end, Constant(F::one()));

    let mut working_arr: Vec<AssignedValue<F>> = Vec::new();
    working_arr.extend(&base_arr); 
    working_arr.push(ctx.load_zero());

    for idx in 0u64..1000 {
        let cur = working_arr[idx as usize];

        let cur_idx = Constant(F::from(idx));
        let left_range = range_gate.is_less_than(ctx, cur_idx, start, range_bits);
        let right_range = range_gate.is_less_than(ctx, end_idx, cur_idx, range_bits);
        let not_in_selected = range_gate.gate().or(ctx, left_range, right_range); 
        let pushed = range_gate.gate().select(ctx, assigned_elt, cur, not_in_selected);

        working_arr[idx as usize] = pushed;
    }


    for itr in 0..1000{
        let cur_iter = ctx.load_witness(F::from(itr));
        let rotate = range_gate.is_less_than(ctx, cur_iter, start, range_bits);
        // let rotate_int = rotate.value().to_u64_limbs(1, 1)[0];
        // working_arr = (0..1000).map(|idx| working_arr[ idx+ (rotate_int as usize)]).collect::<Vec<AssignedValue<F>>>();
        working_arr = (0..1000).map(|idx| range_gate.gate().select(ctx, working_arr[idx+1], working_arr[idx], rotate)).collect::<Vec<AssignedValue<F>>>();
    
        working_arr.push(ctx.load_zero());
    };

    working_arr.pop();
    println!("fin: {:?}", working_arr);

}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    // run different zk commands based on the command line arguments
    run(some_algorithm_in_zk, args);
}