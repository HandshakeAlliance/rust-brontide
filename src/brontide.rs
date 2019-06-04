use crate::cipher_state::CipherState;
use crate::handshake::HandshakeState;

use crate::common::{PROLOGUE, VERSION};

use crate::util::{ecdh, expand, get_public_key};

use secp256k1::PublicKey;

//TODO let's review props in this struct
pub struct Brontide {
    handshake_state: HandshakeState,
    send_cipher: CipherState,
    receive_cipher: CipherState,
}

impl Brontide {
    //TODO review if this is option or not.
    pub fn init(&mut self, initiator: bool, local_pub: [u8; 32], remote_pub: Option<[u8; 32]>) {
        self.handshake_state
            .init_state(initiator, PROLOGUE, local_pub, remote_pub);
    }

    //TODO replace with ACT_ONE Custom type.
    pub fn gen_act_one(&mut self) -> [u8; 50] {
        // e
        self.handshake_state.local_ephemeral = HandshakeState::generate_key();
        let ephemeral = get_public_key(self.handshake_state.local_ephemeral);
        //TODO double check this.
        self.handshake_state.symmetric.mix_digest(&ephemeral, None);

        //ec
        let s = ecdh(
            self.handshake_state.remote_static,
            self.handshake_state.local_ephemeral,
        );
        self.handshake_state.symmetric.mix_key(&s);

        //TODO needs to be an empty buffer of 32 bytes. - Make this a constant when moved to new
        //package
        //TODO decide whether this is 32 0s, or empty.
        let tag = self.handshake_state.symmetric.encrypt_hash(&[]);

        //const ACT_ONE_SIZE = 50;
        // let act_one = Buffer::new();
        let mut act_one = [0_u8; 50];
        act_one[0] = VERSION;
        //Double check this operation TODO
        //Might have to splice from 1..ephemeral.len() + 1
        //Double check this TODO
        act_one[1..33].copy_from_slice(&ephemeral);

        //Double check this operation TODO
        //Might have to splice from 1...tag.len() + 34
        act_one[34..].copy_from_slice(&tag);

        act_one
    }

    //This is going to have to return a Result type to catch errors, TODO
    pub fn recv_act_one(&mut self, act_one: [u8; 50]) {
        if act_one[0] != VERSION {
            //throw error here TODO
            println!("Act one: bad version.");
        }

        //TODO check these operations to ensure proper slicing //inclusive/exclusive etc.
        //TODO also check on the borrowing here, doesn't smell right.
        //I think this is to 33 - double check
        let e = &act_one[1..33];
        //TODO custom type.
        let mut p = [0; 16];
        p.copy_from_slice(&act_one[34..act_one.len()]);

        //We just want to verify here, might be an easier way than creating the actual key.
        //TODO
        let result = PublicKey::from_slice(e);

        if !result.is_ok() {
            //Throw error in here.
            println!("act one: bad key");
        }

        //e
        //TODO code smell
        self.handshake_state.remote_ephemeral.copy_from_slice(e);
        self.handshake_state
            .symmetric
            .mix_digest(&self.handshake_state.remote_ephemeral, None);

        //es
        let s = ecdh(
            self.handshake_state.remote_ephemeral,
            self.handshake_state.local_static,
        );
        self.handshake_state.symmetric.mix_key(&s);

        //TODO must be empty buffer, not new buffer.
        //TODO code smell
        if !self.handshake_state.symmetric.decrypt_hash(&[], p) {
            //throw error
            println!("Act one: bad tag.");
        }
    }

    //TODO custom type return
    pub fn gen_act_two(&mut self) -> [u8; 50] {
        // e
        self.handshake_state.local_ephemeral = HandshakeState::generate_key();

        let ephemeral = get_public_key(self.handshake_state.local_ephemeral);

        self.handshake_state.symmetric.mix_digest(&ephemeral, None);

        // ee
        let s = ecdh(
            self.handshake_state.remote_ephemeral,
            self.handshake_state.local_ephemeral,
        );
        self.handshake_state.symmetric.mix_key(&s);

        //TODO again this needs to be empty buffer, NOT new buffer.
        //TODO empty or 0s?
        let tag = self.handshake_state.symmetric.encrypt_hash(&[]);

        // const ACT_TWO_SIZE = 50;
        let mut act_two = [0_u8; 50];
        act_two[0] = VERSION;

        //TODO all the issues from act one apply here as well, this code needs to be thoroughly
        //checked and tested.
        act_two[1..33].copy_from_slice(&ephemeral);
        act_two[34..].copy_from_slice(&tag);

        act_two
    }

    pub fn recv_act_two(&mut self, act_two: [u8; 50]) {
        if act_two[0] != VERSION {
            //throw error here TODO
            println!("Act two: bad version.");
        }

        //TODO check these operations to ensure proper slicing //inclusive/exclusive etc.
        //TODO also check on the borrowing here, doesn't smell right.
        let e = &act_two[1..34];

        //TODO
        let mut p = [0; 16];

        p.copy_from_slice(&act_two[34..]);

        //We just want to verify here, might be an easier way than creating the actual key.
        //TODO
        let result = PublicKey::from_slice(e);

        if !result.is_ok() {
            //Throw error in here.
            println!("act one: bad key");
        }

        //e
        //TODO code smell
        self.handshake_state.remote_ephemeral.copy_from_slice(e);
        self.handshake_state
            .symmetric
            .mix_digest(&self.handshake_state.remote_ephemeral, None);

        //es
        let s = ecdh(
            self.handshake_state.remote_ephemeral,
            self.handshake_state.local_ephemeral,
        );
        self.handshake_state.symmetric.mix_key(&s);

        //TODO must be empty buffer, not new buffer.
        //TODO code smell
        if !self.handshake_state.symmetric.decrypt_hash(&[], p) {
            //throw error
            println!("Act two: bad tag.");
        }
    }

    //TODO custom act three type
    pub fn gen_act_three(&mut self) -> [u8; 66] {
        let our_pub_key = get_public_key(self.handshake_state.local_static);
        let tag_1 = self.handshake_state.symmetric.encrypt_hash(&our_pub_key);
        let ct = our_pub_key;

        let s = ecdh(
            self.handshake_state.remote_ephemeral,
            self.handshake_state.local_static,
        );
        self.handshake_state.symmetric.mix_key(&s);

        //TODO again must be [u8; 32] empty not new.
        let tag_2 = self.handshake_state.symmetric.encrypt_hash(&[]);

        //const ACT_THREE_SIZE = 66;
        let mut act_three = [0_u8; 66];
        act_three[0] = VERSION;

        //TODO code smell
        act_three[1..33].copy_from_slice(&ct);
        act_three[34..49].copy_from_slice(&tag_1);
        act_three[50..].copy_from_slice(&tag_2);

        self.split();

        act_three
    }

    pub fn recv_act_three(&mut self, act_three: [u8; 66]) {
        if act_three[0] != VERSION {
            //Throw error in here
            println!("Act three: bad version");
        }

        //TODO code smell here...
        let s1 = &act_three[1..34];
        let mut p1 = [0; 16];
        p1.copy_from_slice(&act_three[34..50]);
        let s2 = &act_three[50..50];
        let mut p2 = [0; 16];
        p2.copy_from_slice(&act_three[50..66]);

        // s
        if self.handshake_state.symmetric.decrypt_hash(s1, p1) {
            //Throw error
            println!("act three: bad tag");
        }

        let remote_public = s1;

        let result = PublicKey::from_slice(&remote_public);

        if result.is_err() {
            //Throw error here TODO
            println!("act three: bad key.");
        }

        self.handshake_state
            .remote_static
            .copy_from_slice(remote_public);

        // se
        let se = ecdh(
            self.handshake_state.remote_static,
            self.handshake_state.local_ephemeral,
        );
        self.handshake_state.symmetric.mix_key(&se);

        if self.handshake_state.symmetric.decrypt_hash(s2, p2) {
            //Throw error, bad tag
            println!("act three bad tag");
        }

        self.split();
    }

    //TODO write and read
    //pub fn write(data: [u8; 32]) {
    //    if data.len() <= 0xffff {
    //        //throw error -> Not sure what yet though TODO
    //    }

    //    //Needs to be a packet of length 2 + 16 + data.len() + 16
    //    //TODO I think this is correct
    //    let packet = Vec<u8>;

    //}

    //TODO review thoroughly AND TEST
    pub fn split(&mut self) {
        //TODO must be buffer empty not new
        let (h1, h2) = expand(&[], &self.handshake_state.symmetric.chaining_key);

        if self.handshake_state.initiator {
            let send_key = h1;
            self.send_cipher =
                CipherState::new(send_key, self.handshake_state.symmetric.chaining_key);
            let recv_key = h2;
            self.receive_cipher =
                CipherState::new(recv_key, self.handshake_state.symmetric.chaining_key);
        } else {
            let recv_key = h1;
            self.receive_cipher =
                CipherState::new(recv_key, self.handshake_state.symmetric.chaining_key);
            let send_key = h2;
            self.send_cipher =
                CipherState::new(send_key, self.handshake_state.symmetric.chaining_key);
        }
    }
}