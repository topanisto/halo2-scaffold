use clap::Parser;
use halo2_base::gates::{GateInstructions, RangeChip, RangeInstructions};
use halo2_base::halo2_proofs::arithmetic::compute_inner_product;
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

    let num_bits: usize = 16;
    let range: RangeChip<F> = RangeChip::default(lookup_bits);

    // check r is 16-bit
    range.range_check(ctx, x, num_bits);
    println!("x is {} bits!", num_bits);

    //without num_to_bits-- keep on subtracting powers of 2?

    let mut x_rem = range.gate().add(ctx, Constant(F::zero()), x);

    let mut pow2 = (5..16).map(|i| Constant(range.gate().pow_of_two()[i])).collect::<Vec<QuantumCell<F>>>();

    for _ in 0..11 {
        let cur_pow = pow2.pop().unwrap();
        let valid_sub = range.is_less_than(ctx, x_rem, cur_pow, num_bits);
        let sub_val = range.gate().select(ctx, Constant(F::zero()), cur_pow, valid_sub);
        x_rem = range.gate().sub(ctx, x_rem, sub_val);
    };

    let x_quot = range.gate().sub(ctx, x, x_rem);

    let x_out = range.gate().div_unsafe(ctx, x_quot, Constant(F::from(32)));

    make_public.push(x_out);
    print!("quotient: {:?}", x_out.value());




    // num_to_bits implementations

    let mut x_bitw = range.gate().num_to_bits(ctx, x, num_bits);

    let mut cumsum = ctx.load_witness(F::zero());

    let two = Constant(F::from(2));

    for _ in 0..11 {
        let cur_const = x_bitw.pop().unwrap();
        // going down the vector backwards

        // since bits are popped backwards, we check if we are still in the bracket for 2^k, where k>5
        // our output is either 2*cumsum + cur_const or 1*cumsum + 0
        let new = range.gate().mul_add(ctx, two, cumsum, cur_const);
        cumsum = new;
    };

    // num_to_bits without loops

    let mut fe = (0..5).map(|_| Constant(F::zero())).collect::<Vec<QuantumCell<F>>>();
    fe.extend((5..16).map(|i| Constant(range.gate().pow_of_two()[i])).collect::<Vec<QuantumCell<F>>>());

    let _cumsum = range.gate().inner_product(ctx, x_bitw, fe);

    // make_public.push(cumsum);
    // println!("final: {:?}", cumsum.value());

}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    // run different zk commands based on the command line arguments
    run(div32, args);
}
