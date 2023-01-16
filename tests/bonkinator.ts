import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Bonkinator } from "../target/types/bonkinator";

describe("bonkinator", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Bonkinator as Program<Bonkinator>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
