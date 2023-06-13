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

    let range_bits = 16;
    let range: RangeChip<F> = RangeChip::default(lookup_bits);

    // check r is 16-bit
    range.range_check(ctx, x, range_bits);
    println!("x is {} bits!", range_bits);

    let (mod32, _rem) = range.div_mod(ctx, x, 32_u64, range_bits);

    println!("mod_32: {:?}", mod32.value());

}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    // run different zk commands based on the command line arguments
    run(div32, args);
}
