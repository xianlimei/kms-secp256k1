/*
    KMS-ECDSA

    Copyright 2018 by Kzen Networks

    This file is part of KMS library
    (https://github.com/KZen-networks/kms)

    Cryptography utilities is free software: you can redistribute
    it and/or modify it under the terms of the GNU General Public
    License as published by the Free Software Foundation, either
    version 3 of the License, or (at your option) any later version.

    @license GPL-3.0+ <https://github.com/KZen-networks/kms/blob/master/LICENSE>
*/
use super::traits::ManagementSystem;

use cryptography_utils::cryptographic_primitives::twoparty::coin_flip_optimal_rounds;
use cryptography_utils::{BigInt, FE, GE};
use multi_party_ecdsa::protocols::two_party_ecdsa::lindell_2017::{party_one, party_two};
use paillier::proof::Challenge;
//TODO: pick only relevant
use cryptography_utils::arithmetic::traits::Modulo;
use cryptography_utils::cryptographic_primitives::twoparty::dh_key_exchange;
use cryptography_utils::elliptic::curves::secp256_k1::Secp256k1Scalar;
use cryptography_utils::elliptic::curves::traits::ECPoint;
use cryptography_utils::elliptic::curves::traits::ECScalar;
use paillier::*;
use std::borrow::Cow;

pub struct Party2Public<'a> {
    pub Q: GE,
    pub P2: GE,
    paillier_pub: EncryptionKey,
    c_key: RawCiphertext<'a>,
}

struct Party2Private {
    x2: FE,
}

pub struct MasterKey2<'a> {
    toggle: bool,
    public: Party2Public<'a>,
    private: Party2Private,
    chain_code: BigInt,
}

impl<'a> ManagementSystem<Party2Public<'a>, Party2Private> for MasterKey2<'a> {
    fn rotate(&mut self, cf: &BigInt) -> &mut Self {
        let rand_str: FE = ECScalar::from_big_int(cf);
        let rand_str_invert: FE = ECScalar::from_big_int(&cf.invert(&rand_str.get_q()).unwrap());
        //TODO: use proper set functions
        self.private.x2 = self.private.x2.mul(&rand_str_invert.get_element());
        self.public.P2 = self
            .public
            .P2
            .clone()
            .scalar_mul(&rand_str_invert.get_element());
        let c_key_new = BigInt::mod_pow(&self.public.c_key.0, cf, &rand_str.get_q());
        self.public.c_key = RawCiphertext(Cow::Owned(c_key_new));
        return self;
    }

    //fn get_child(&self, index: BigInt, height: BigInt) -> (Party2Public, Party2Private) {}
}

impl<'a> MasterKey2<'a> {
    pub fn chain_code_first_message() -> dh_key_exchange::Party2FirstMessage {
        dh_key_exchange::Party2FirstMessage::create()
    }
    pub fn chain_code_second_message(
        party_one_first_message: &dh_key_exchange::Party1FirstMessage,
        party_one_second_message: &dh_key_exchange::Party1SecondMessage,
    ) -> dh_key_exchange::Party2SecondMessage {
        dh_key_exchange::Party2SecondMessage::verify_commitments_and_dlog_proof(
            &party_one_first_message.pk_commitment,
            &party_one_first_message.zk_pok_commitment,
            &party_one_second_message.zk_pok_blind_factor,
            &party_one_second_message.public_share,
            &party_one_second_message.pk_commitment_blind_factor,
            &party_one_second_message.d_log_proof,
        ).expect("")
    }

    pub fn compute_chain_code(
        first_message: &dh_key_exchange::Party1FirstMessage,
        party2_first_message: &dh_key_exchange::Party2FirstMessage,
    ) -> GE {
        dh_key_exchange::compute_pubkey_party2(party2_first_message, first_message)
    }

    pub fn key_gen_first_message() -> party_two::KeyGenFirstMsg {
        party_two::KeyGenFirstMsg::create()
    }
    pub fn key_gen_second_message(
        party_one_first_message: &party_one::KeyGenFirstMsg,
        party_one_second_message: &party_one::KeyGenSecondMsg,
    ) -> party_two::KeyGenSecondMsg {
        party_two::KeyGenSecondMsg::verify_commitments_and_dlog_proof(
            &party_one_first_message.pk_commitment,
            &party_one_first_message.zk_pok_commitment,
            &party_one_second_message.zk_pok_blind_factor,
            &party_one_second_message.public_share,
            &party_one_second_message.pk_commitment_blind_factor,
            &party_one_second_message.d_log_proof,
        ).expect("")
    }

    pub fn key_gen_third_message(
        party_two_paillier: &party_two::PaillierPublic,
    ) -> (Challenge, VerificationAid) {
        party_two::PaillierPublic::generate_correct_key_challenge(&party_two_paillier)
    }
    //TODO: move serialization to this code base from signature code base.
    pub fn key_gen_forth_message(
        party_two_paillier: &party_two::PaillierPublic,
        challenge: &ChallengeBits,
        encrypted_pairs: &EncryptedPairs,
        proof: &Proof,
    ) -> bool {
        party_two::PaillierPublic::verify_range_proof(
            &party_two_paillier,
            &challenge,
            &encrypted_pairs,
            &proof,
        )
    }

    pub fn key_rotate_first_message(
        party1_first_message: &coin_flip_optimal_rounds::Party1FirstMessage,
    ) -> (coin_flip_optimal_rounds::Party2FirstMessage) {
        coin_flip_optimal_rounds::Party2FirstMessage::share(&party1_first_message.proof)
    }

    pub fn key_rotate_second_message(
        party1_second_message: &coin_flip_optimal_rounds::Party1SecondMessage,
        party2_first_message: &coin_flip_optimal_rounds::Party2FirstMessage,
        party1_first_message: &coin_flip_optimal_rounds::Party1FirstMessage,
    ) -> Secp256k1Scalar {
        coin_flip_optimal_rounds::finalize(
            &party1_second_message.proof,
            &party2_first_message.seed,
            &party1_first_message.proof.com,
        )
    }
}
