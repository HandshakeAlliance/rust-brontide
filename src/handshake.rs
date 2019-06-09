use crate::common::PROTOCOL_NAME;
use crate::symmetric_state::SymmetricState;
use hex;
use secp256k1::rand::rngs::OsRng;
use secp256k1::{PublicKey, Secp256k1, SecretKey};

//TODO review pub exposure
pub struct HandshakeState {
    pub(crate) symmetric: SymmetricState,
    pub(crate) initiator: bool,
    //TODO again all these should be custom
    pub(crate) local_static: [u8; 32],
    pub(crate) local_ephemeral: [u8; 32],
    //TODO both these need to be 33.
    pub(crate) remote_static: [u8; 33],
    pub(crate) remote_ephemeral: [u8; 33],
    pub generate_key: fn() -> [u8; 32],
}

impl HandshakeState {
    //pub fn generate_key() -> [u8; 32] {
    //    let secp = Secp256k1::new();
    //    let mut rng = OsRng::new().expect("OsRng");
    //    let (secret_key, _) = secp.generate_keypair(&mut rng);

    //    //TODO redo this.
    //    let mut key = [0_u8; 32];
    //    //TODO capture this unwrap and handle accordling
    //    key.copy_from_slice(&hex::decode(secret_key.to_string()).unwrap());

    //    key
    //}

    pub(crate) fn new(
        initiator: bool,
        prologue: &str,
        local_pub: [u8; 32],
        remote_pub: Option<[u8; 33]>,
    ) -> Self {
        let remote_public_key: [u8; 33];

        if let Some(remote_pub_ok) = remote_pub {
            remote_public_key = remote_pub_ok
        } else {
            //Should be zero key not buffer new, TODO
            remote_public_key = [0_u8; 33];
        }

        //Need constants here TODO
        //ZERO_PUB
        //ZERO_KEY

        let mut state = HandshakeState {
            initiator,
            local_static: local_pub,
            remote_static: remote_public_key,
            symmetric: SymmetricState::new(PROTOCOL_NAME),
            local_ephemeral: [0; 32],
            remote_ephemeral: [0; 33],
            generate_key: || {
                let secp = Secp256k1::new();
                let mut rng = OsRng::new().expect("OsRng");
                let (secret_key, _) = secp.generate_keypair(&mut rng);

                //TODO redo this.
                let mut key = [0_u8; 32];
                //TODO capture this unwrap and handle accordling
                key.copy_from_slice(&hex::decode(secret_key.to_string()).unwrap());

                key
            },
        };

        state.symmetric.mix_digest(prologue.as_bytes(), None);

        //TODO review this logic.
        if initiator {
            state.symmetric.mix_digest(&state.remote_static, None);
        } else {
            let secp = Secp256k1::new();

            let secret_key =
                SecretKey::from_slice(&state.local_static).expect("32 bytes, within curve order");
            let public_key = PublicKey::from_secret_key(&secp, &secret_key);

            state
                .symmetric
                .mix_digest(&hex::decode(public_key.to_string()).unwrap(), None);
        }

        state
    }

    //TODO remove this.
    pub fn init_state(
        &mut self,
        initiator: bool,
        prologue: &str,
        local_pub: [u8; 32],
        remote_pub: Option<[u8; 33]>,
    ) {
        let remote_public_key: [u8; 33];
        self.initiator = initiator;
        //TODO might not have to do this.
        self.local_static.copy_from_slice(&local_pub);
        if let Some(remote_pub_ok) = remote_pub {
            remote_public_key = remote_pub_ok
        } else {
            //Should be zero key not buffer new, TODO
            remote_public_key = [0_u8; 33];
        }

        self.remote_static = remote_public_key;

        self.symmetric = SymmetricState::new(PROTOCOL_NAME);

        //Might have to make sure this works as ascii TODO
        self.symmetric.mix_digest(prologue.as_bytes(), None);

        if initiator {
            //TODO we need to test this behavior, but I think the general idea is we want to mix
            //this with a zero hash buffer. so 32 bytes of 0s.
            self.symmetric.mix_digest(&remote_public_key, None)
        } else {
            //Switch this with the get public function TODO
            let secp = Secp256k1::new();
            //TODO handle this error correctly.
            let secret_key =
                SecretKey::from_slice(&local_pub).expect("32 bytes, within curve order");
            let public_key = PublicKey::from_secret_key(&secp, &secret_key);
            //TODO review this, not sure I trust converting the public key to string then reading
            //it in the buffer.
            self.symmetric
                .mix_digest(public_key.to_string().as_bytes(), None);
        }
    }
}
