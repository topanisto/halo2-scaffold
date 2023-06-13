use clap::Parser;
use halo2_base::gates::{RangeChip, RangeInstructions, GateInstructions};
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
    pub arr: [String; 20],
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

    let base_arr = input.arr.map(|x| ctx.load_witness(F::from_str_vartime(&x).unwrap()));
    make_public.extend_from_slice(&base_arr);
    //array values are public now

    let start = F::from_str_vartime(&input.start).expect("deserialize field element should not fail");
    let end = F::from_str_vartime(&input.end).expect("deserialize field element should not fail");


    let start = ctx.load_witness(start);
    make_public.push(start);

    let end = ctx.load_witness(end);
    make_public.push(end);

    let lookup_bits =
        var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();

    let rgate: RangeChip<F> = RangeChip::default(lookup_bits);

    let dif = rgate.gate().sub(ctx, end, start);
    let range_bits = 16;

    let arr_len = Constant(F::from(20));

    let mut fin: Vec<AssignedValue<F>> = Vec::new();

    // running array idxes
    let mut cur = F::zero();
    let mut cur_q = ctx.load_witness(cur);
    let fil_const = Constant(F::zero());
    let fil_cell = ctx.assign_region_last([fil_const, Constant(F::zero()), Constant(F::zero()), fil_const], [0]);

    while rgate.is_less_than(ctx, cur_q, arr_len, range_bits).value() == &F::one() {

        while rgate.is_less_than(ctx, cur_q, dif, range_bits).value() == &F::one() {
            let from_cur_idx = rgate.gate().select_from_idx(ctx, base_arr, cur_q);
            make_public.push(from_cur_idx);
            fin.push(from_cur_idx);

            cur += F::one();
            cur_q = ctx.load_witness(cur);
            make_public.push(cur_q);
        }

        let fin_fill = ctx.assign_region_last([Existing(fil_cell), Constant(F::zero()), Constant(F::zero()), Existing(fil_cell)], [0]);
        
        fin.push(fin_fill);
        make_public.push(cur_q);
        cur += F::one();
        cur_q = ctx.load_witness(cur);
        make_public.push(cur_q);
    }

    let fin_vals = fin.iter().map(|x| x.value()).collect::<Vec<&F>>();


    println!("Final array: {:?}", fin_vals);

}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    // run different zk commands based on the command line arguments
    run(some_algorithm_in_zk, args);
}
