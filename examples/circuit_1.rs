use clap::Parser;
use halo2_base::gates::{GateChip, GateInstructions};
use halo2_base::utils::ScalarField;
use halo2_base::AssignedValue;
#[allow(unused_imports)]
use halo2_base::{
    Context,
    QuantumCell::{Constant, Existing, Witness},
};
use halo2_scaffold::scaffold::cmd::Cli;
use halo2_scaffold::scaffold::run;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    // 3 field elements for pythagorean triple
    pub inputs: [String; 3], // field element, but easier to deserialize as a string
}

// this algorithm takes a public input x, computes x^2 + 72, and outputs the result as public output
fn some_algorithm_in_zk<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>, //push into this values we want to be public
) {

    let [x, y, z] = input.inputs.map(|x:String| ctx.load_witness(F::from_str_vartime(&x).unwrap()));
    make_public.extend([x, y]);
    // z is a private witness value

    // `Context` can roughly be thought of as a single-threaded execution trace of a program we want to ZK prove.
    //   We do some post-processing on `Context` to optimally divide the execution trace into multiple columns in a
    //   PLONKish arithmetization
    // More advanced usage with multi-threaded witness generation is possible, but we do not explain it here

    // create a Gate chip that contains methods for basic arithmetic operations
    let gate = GateChip::<F>::default();

    // ===== way 1 =====
    // now we can perform arithmetic operations almost like a normal program using halo2-lib API functions
    // square x
    let z: F = *z.value();
    let x_sq: AssignedValue<F> = gate.mul(ctx, x, x);
    let y_sq: AssignedValue<F> = gate.mul(ctx, y, y);
    let xsq_plus_ysq = gate.add(ctx, x_sq, y_sq);
    let _val_assigned = ctx.assign_region_last([Constant(F::zero()), Witness(z), Witness(z), Existing(xsq_plus_ysq)], [0]);

    assert_eq!(*xsq_plus_ysq.value(), z * z);
    println!("x val: {:?}", x.value());
    println!("y val: {:?}", y.value());
    // println!("z val: {:?}", z.value());

    //assert that x_sq + y_sq is equal to the square of some witness

}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    // run different zk commands based on the command line arguments
    run(some_algorithm_in_zk, args);
}
