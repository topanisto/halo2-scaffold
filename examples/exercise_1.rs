use clap::Parser;
use halo2_base::gates::{GateInstructions, RangeChip, RangeInstructions};
use halo2_base::halo2_proofs::arithmetic::compute_inner_product;
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
    pub x: String,

}

// this algorithm takes a public input x, computes x^2 + 72, and outputs the result as public output
fn div32<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>, //push into this values we want to be public
) {

    let x: F = F::from_str_vartime(&input.x).expect("Something went wrong");
    let x = ctx.load_witness(x);
    make_public.push(x);

    let lookup_bits =
        var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();

    let range_bits: usize = 16;
    let range: RangeChip<F> = RangeChip::default(lookup_bits);

    // check r is 16-bit
    range.range_check(ctx, x, range_bits);
    println!("x is {} bits!", range_bits);

    let mut x_bitw = range.gate().num_to_bits(ctx, x, range_bits);


    let mut cumsum = ctx.load_witness(F::zero());


    let two = Constant(F::from(2));
    let last_power = range.gate().sub(ctx, Constant(F::from(16)), Constant(F::from(5)));

    for pow in 0..16 {
        let cur_const = x_bitw.pop().unwrap();
        // going down the vector backwards

        let idx = ctx.load_witness(F::from(pow));
        let is_quotient = range.is_less_than(ctx, idx, last_power, range_bits);
        // since bits are popped backwards, we check if we are still in the bracket for 2^k, where k>5

        let multiplier = range.gate().select(ctx, Constant(F::one()), Constant(F::zero()), is_quotient); //const multiplier
        let power_of_two = range.gate().select(ctx, two, Constant(F::one()), is_quotient);
        // our output is either 2*cumsum + cur_const or 1*cumsum + 0

        let cur_zeroth = range.gate().mul(ctx, multiplier, cur_const);
        let new = range.gate().mul_add(ctx, power_of_two, cumsum, cur_zeroth);
        cumsum = new;
    }

    make_public.push(cumsum);

    println!("final: {:?}", cumsum.value());


    // println!("mod_32: {:?}", mod32.value());

}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    // run different zk commands based on the command line arguments
    run(div32, args);
}
