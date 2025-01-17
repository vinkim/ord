use {
  super::*,
  base64::{engine::general_purpose, Engine},
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub address: Address<NetworkUnchecked>,
  pub message: String,
  pub witness: String,
}

#[derive(Debug, Parser)]
pub(crate) struct Sign {
  #[arg(long, help = "Sign for <ADDRESS>.")]
  address: Address<NetworkUnchecked>,
  #[arg(long, help = "Sign <MESSAGE>.")]
  message: String,
}

impl Sign {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    let address = self.address.require_network(wallet.chain().network())?;

    let to_spend = bip322::create_to_spend(&address, self.message.as_bytes())?;

    let to_sign = bip322::create_to_sign(&to_spend, None)?;

    let result = wallet.bitcoin_client().sign_raw_transaction_with_wallet(
      &to_sign.extract_tx()?,
      Some(&[bitcoincore_rpc::json::SignRawTransactionInput {
        txid: to_spend.compute_txid(),
        vout: 0,
        script_pub_key: address.script_pubkey(),
        redeem_script: None,
        amount: Some(Amount::ZERO),
      }]),
      None,
    )?;

    let mut buffer = Vec::new();

    Transaction::consensus_decode(&mut result.hex.as_slice())?.input[0]
      .witness
      .consensus_encode(&mut buffer)?;

    Ok(Some(Box::new(Output {
      address: address.as_unchecked().clone(),
      message: self.message,
      witness: general_purpose::STANDARD.encode(buffer),
    })))
  }
}
