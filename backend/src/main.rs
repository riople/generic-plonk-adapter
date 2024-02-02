use clap::{command, Parser, Subcommand};
use powdr::Pipeline;
use acvm::acir::{circuit::{Circuit, Opcode}, native_types::{Expression, WitnessMap}};
use powdr::Bn254Field;

use std::{marker::PhantomData, io};
use std::fs;
use std::path::Path;
use std::io::Read;

// Noir field element
use acvm::acir::acir_field::FieldElement;
use acvm::acir::native_types::Witness;

const noir_default_arithmetic: &str = r#"
        pol commit a, b, c;
        pol commit q_l, q_r, q_o, q_m, q_c;

        // q_add * (a + b) + q_mul * (a * b) - c = 0;
        // Standard plonk constraints
        ( q_l * a ) + ( q_r * b ) + ( q_o * c ) + ( q_m * (a * b) ) + ( q_c * c ) = 0;
  "#;

  #[derive(Parser, Debug)]
  #[command(author, version, about, long_about = None)]
  struct PlonkAdapter {
      #[arg(short, long, global = true)]
      crs: Option<String>,
  
      #[arg(short, long, global = true)]
      bytecode: Option<String>,
  
      #[arg(short, long, global = true)]
      output: Option<String>,
  
      #[clap(subcommand)]
      command: Commands,
  }
  
  #[derive(Subcommand, Debug)]
  enum Commands {
      Prove(Prove),
      Verify,
      Version,
      VkAsFields,
      PkAsFields,
      WriteVk,
      WritePk,
      Contract,
      Gates,
      Info(Info)
  }
  
  #[derive(Parser, Debug)]
  struct Prove {
      #[arg(short, long)]
      vk: Option<String>,
      #[arg(short, long)]
      pk: Option<String>,
      #[arg(short, long, global = true)]
      witness: Option<String>,
      // #[clap(short, long)]
      // proof: String,
      // #[clap(short, long)]
      // inputs: String,
      // #[clap(short, long)]
      // output: String,
      // #[clap(short, long)]
      // verbose: bool,
  
  }

#[derive(Parser, Debug)]
struct Info {

}

struct GlobalOptions {
    // crs: Option<String>,
    bytecode: Option<String>,
    output: Option<String>,
}

fn main() {
    let args = PlonkAdapter::parse();
    let command = args.command;

    let globals = GlobalOptions {
        // crs: args.crs,
        bytecode: args.bytecode,
        output: args.output,
    };

    // TODO: package up the globals better
    match command {
        Commands::Prove(prove_cmd) => prove(prove_cmd, globals),
        // Commands::Verify => verify(),
        // Halo2Backend::Version => version(),
        // Halo2Backend::VkAsFields => vk_as_fields(),
        // Halo2Backend::PkAsFields => pk_as_fields(),
        // Halo2Backend::WriteVk => write_vk(),
        // Halo2Backend::WritePk => write_pk(),
        // Halo2Backend::Contract => contract(),
        // Halo2Backend::Gates => gates(),
        Commands::Info(info_cmd) => info(info_cmd),
        _ => todo!("lazy shite")
    }
}

fn prove(prove_cmd: Prove, globals: GlobalOptions) {
    // Read the circuit file
    let circuit_path = globals.bytecode.unwrap_or("./target/acir.gz".to_owned());
    let _output_path = globals.output.unwrap_or("./target/proof.gz".to_owned());

    let witness_path = prove_cmd.witness.unwrap_or("./target/witness.gz".to_owned());
    let _vk_path = prove_cmd.vk.unwrap_or("./target/vk.gz".to_owned());
    let _pk_path = prove_cmd.pk.unwrap_or("./target/pk.gz".to_owned());

    let circuit_bytes = std::fs::read(circuit_path).unwrap();
    let circuit = Circuit::deserialize_circuit(&*circuit_bytes).expect("failed to read circuit");
    
    let witness_bytes = std::fs::read(witness_path).unwrap();
    let witness_values = WitnessMap::try_from(&*witness_bytes).expect("failed to read witness");

    // The plan
    // - We want to first be able to represent arithmetic expressions with no copies, build a
    // simple circuit builder in rust
    // - We can have the example pil file for the meantime
    // - Then we can make this be a fixed circuit first
    // - We can build more complicated circuits as time goes on
    //

    println!("Circuit: {:?}", circuit);

    // Make a test circuit that we will injest
    let mut pipeline: Pipeline<Bn254Field> = Pipeline::default().from_pil_string(noir_default_arithmetic.to_owned());

    pipeline.advance_to(powdr::pipeline::Stage::AnalyzedPil).unwrap();

    let pil_artifact = pipeline.optimized_pil().unwrap();

    let fixed_columns = pil_artifact.constant_polys_in_source_order();
    let witness_columns = pil_artifact.committed_polys_in_source_order();
    println!("Witness: {:?}", witness_columns);
    println!("Fixed: {:?}", fixed_columns);

    let identities = pil_artifact.identities_with_inlined_intermediate_polynomials();
    println!("identities: {:?}", identities);

    // With the columns provided, we can now use the circuit builder



}

struct PlonkDefault {
    a: Witness,
    b: Witness,
    c: Witness,
    ql: FieldElement,
    qr: FieldElement,
    qo: FieldElement,
    qm: FieldElement,
    qc: FieldElement,
}

impl Default for PlonkDefault {
    fn default() -> Self {
        Self {
            a: Witness(0),
            b: Witness(0),
            c: Witness(0),
            ql: FieldElement::zero(),
            qr: FieldElement::zero(),
            qo: FieldElement::zero(),
            qm: FieldElement::zero(),
            qc: FieldElement::zero(),
        }
    }
}

impl PlonkDefault {
    pub(crate) fn set_linear_term(&mut self, x: FieldElement, witness: Witness) {
        if self.a == Witness(0) || self.a == witness {
            self.a = witness;
            self.ql = x;
        } else if self.b == Witness(0) || self.b == witness {
            self.b = witness;
            self.qr = x;
        } else if self.c == Witness(0) || self.c == witness {
            self.c = witness;
            self.qo = x;
        } else {
            unreachable!("Cannot assign linear term to a constrain of width 3");
        }
    }
}

struct Program {
    // The circuit
    circuit: Circuit,
    // The witness
    witness: WitnessMap,
}

impl Program {
    pub fn new(circuit: Circuit, witness: WitnessMap) -> Self {
        Self {
            circuit,
            witness,
        }
    }

    pub fn build_circuit(&self) {
        for opcode in &self.circuit.opcodes {
            match opcode {
                Opcode::AssertZero(expression) => self.build_arithmetic_gate(expression),
                _ => todo!("no opcodo understando")
            }
        }
    }

fn build_arithmetic_gate(&self, gate: Expression) {

    let mut noir_cs = PlonkDefault::default();
    // check mul gate
    if !gate.mul_terms.is_empty() {
        let mul_term = &gate.mul_terms[0];
        noir_cs.qm = mul_term.0;

        // Get wL term
        noir_cs.a = mul_term.1;

        // Get wR term
        noir_cs.b = mul_term.2;
    }

    for term in &gate.linear_combinations {
        noir_cs.set_linear_term(term.0, term.1);
    }

    // Add the qc term
    noir_cs.qc = gate.q_c;

    let a = self.witness.get(&noir_cs.a).unwrap_or(&FieldElement::zero());
    let b = self.witness.get(&noir_cs.b).unwrap_or(&FieldElement::zero());
    let c = self.witness.get(&noir_cs.c).unwrap_or(&FieldElement::zero());



}
}



fn build_arithmetic_gate(gate: Expression) {

    // bool a_set = false;
    // bool b_set = false;
    // bool c_set = false;

    // // If necessary, set values for quadratic term (q_m * w_l * w_r)
    // ASSERT(arg.mul_terms.size() <= 1); // We can only accommodate 1 quadratic term
    // // Note: mul_terms are tuples of the form {selector_value, witness_idx_1, witness_idx_2}
    // if (!arg.mul_terms.empty()) {
    //     const auto& mul_term = arg.mul_terms[0];
    //     pt.q_m = uint256_t(std::get<0>(mul_term));
    //     pt.a = std::get<1>(mul_term).value;
    //     pt.b = std::get<2>(mul_term).value;
    //     a_set = true;
    //     b_set = true;
    // }

    // // If necessary, set values for linears terms q_l * w_l, q_r * w_r and q_o * w_o
    // ASSERT(arg.linear_combinations.size() <= 3); // We can only accommodate 3 linear terms
    // for (const auto& linear_term : arg.linear_combinations) {
    //     bb::fr selector_value(uint256_t(std::get<0>(linear_term)));
    //     uint32_t witness_idx = std::get<1>(linear_term).value;

    //     // If the witness index has not yet been set or if the corresponding linear term is active, set the witness
    //     // index and the corresponding selector value.
    //     // TODO(https://github.com/AztecProtocol/barretenberg/issues/816): May need to adjust the pt.a == witness_idx
    //     // check (and the others like it) since we initialize a,b,c with 0 but 0 is a valid witness index once the
    //     // +1 offset is removed from noir.
    //     if (!a_set || pt.a == witness_idx) { // q_l * w_l
    //         pt.a = witness_idx;
    //         pt.q_l = selector_value;
    //         a_set = true;
    //     } else if (!b_set || pt.b == witness_idx) { // q_r * w_r
    //         pt.b = witness_idx;
    //         pt.q_r = selector_value;
    //         b_set = true;
    //     } else if (!c_set || pt.c == witness_idx) { // q_o * w_o
    //         pt.c = witness_idx;
    //         pt.q_o = selector_value;
    //         c_set = true;
    //     } else {
    //         throw_or_abort("Cannot assign linear term to a constraint of width 3");
    //     }
    // }

    // // Set constant value q_c
    // pt.q_c = uint256_t(arg.q_c);
    // return pt;
    // Default arithmetic gate
}


// TODO: make gooder
fn info(_info_cmd: Info){
    println!("{{ \"language\": {{ \"name\": \"PLONK-CSAT\", \"width\": 3 }}, \"opcodes_supported\": [], \"black_box_functions_supported\": [] }}");
}

  

// use powdr_pipeline::{Pipeline, Stage, verify, BackendType, test_util::resolve_test_file};
// use std::path::PathBuf;
// use powdr_number::GoldilocksField;
//
// let mut pipeline = Pipeline::<GoldilocksField>::default()
//   .from_file(resolve_test_file("pil/fibonacci.pil"))
//   .with_backend(BackendType::PilStarkCli);
//
// Advance to some stage (which might have side effects)
// pipeline.advance_to(Stage::OptimizedPil).unwrap();
//
// // Get the result
// let proof = pipeline.proof().unwrap();

