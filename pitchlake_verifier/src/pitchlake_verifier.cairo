#[starknet::interface]
pub trait IPitchLakeVerifier<TContractState> {
    fn verify_proof(
        ref self: TContractState, proof: Span<felt252>, ipfs_hash: ByteArray, is_build: bool,
    );
    fn update_verifier_address(
        ref self: TContractState, new_verifier_address: starknet::ContractAddress,
    );
    fn get_groth16_verifier_address(self: @TContractState) -> starknet::ContractAddress;
    fn get_pitchlake_client_address(self: @TContractState) -> starknet::ContractAddress;
    fn upgrade(ref self: TContractState, new_class_hash: starknet::ClassHash);
}

#[starknet::interface]
pub trait IFossilClient<TContractState> {
    fn fossil_callback(ref self: TContractState, result: Span<felt252>);
}

#[starknet::contract]
mod PitchLakeVerifier {
    use openzeppelin_access::ownable::OwnableComponent;
    use openzeppelin_upgrades::UpgradeableComponent;
    use pitchlake_verifier::decode_journal;
    use pitchlake_verifier::groth16_verifier::{
        IRisc0Groth16VerifierBN254Dispatcher, IRisc0Groth16VerifierBN254DispatcherTrait,
    };
    use super::{IFossilClientDispatcher, IFossilClientDispatcherTrait};

    component!(path: OwnableComponent, storage: ownable, event: OwnableEvent);
    component!(path: UpgradeableComponent, storage: upgradeable, event: UpgradeableEvent);

    // Ownable Mixin
    #[abi(embed_v0)]
    impl OwnableMixinImpl = OwnableComponent::OwnableMixinImpl<ContractState>;
    impl OwnableInternalImpl = OwnableComponent::InternalImpl<ContractState>;

    // Upgradeable
    impl UpgradeableInternalImpl = UpgradeableComponent::InternalImpl<ContractState>;


    #[storage]
    struct Storage {
        bn254_verifier: IRisc0Groth16VerifierBN254Dispatcher,
        pitchlake_client: IFossilClientDispatcher,
        #[substorage(v0)]
        ownable: OwnableComponent::Storage,
        #[substorage(v0)]
        upgradeable: UpgradeableComponent::Storage,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        PitchlakeProofVerified: PitchlakeProofVerified,
        #[flat]
        OwnableEvent: OwnableComponent::Event,
        #[flat]
        UpgradeableEvent: UpgradeableComponent::Event,
    }

    #[derive(Drop, starknet::Event)]
    struct PitchlakeProofVerified {
        data_8_months_hash: [u32; 8],
        start_timestamp: u64,
        end_timestamp: u64,
        reserve_price: felt252,
        floating_point_tolerance: felt252,
        reserve_price_tolerance: felt252,
        twap_tolerance: felt252,
        gradient_tolerance: felt252,
        twap_result: felt252,
        max_return: felt252,
    }

    #[constructor]
    fn constructor(
        ref self: ContractState,
        verifier_address: starknet::ContractAddress,
        fossil_store_address: starknet::ContractAddress,
        owner: starknet::ContractAddress,
    ) {
        self
            .bn254_verifier
            .write(IRisc0Groth16VerifierBN254Dispatcher { contract_address: verifier_address });
        self
            .pitchlake_client
            .write(IFossilClientDispatcher { contract_address: fossil_store_address });
        self.ownable.initializer(owner);
    }

    #[abi(embed_v0)]
    impl PitchLakeVerifierImpl of super::IPitchLakeVerifier<ContractState> {
        fn verify_proof(
            ref self: ContractState, mut proof: Span<felt252>, ipfs_hash: ByteArray, is_build: bool,
        ) {
            let _ = proof.pop_front();
            let journal = self
                .bn254_verifier
                .read()
                .verify_groth16_proof_bn254(proof)
                .expect('Failed to verify proof');

            let journal = decode_journal(journal);

            let mut calldata: Array<felt252> = array![];

            // TODO: review the Journal fields that need to be sent to pitchlake client
            journal.start_timestamp.serialize(ref calldata);

            let pitchlake_client = self.pitchlake_client.read();

            pitchlake_client.fossil_callback(calldata.span());

            self
                .emit(
                    PitchlakeProofVerified {
                        data_8_months_hash: journal.data_8_months_hash,
                        start_timestamp: journal.start_timestamp,
                        end_timestamp: journal.end_timestamp,
                        reserve_price: journal.reserve_price,
                        floating_point_tolerance: journal.floating_point_tolerance,
                        reserve_price_tolerance: journal.reserve_price_tolerance,
                        twap_tolerance: journal.twap_tolerance,
                        gradient_tolerance: journal.gradient_tolerance,
                        twap_result: journal.twap_result,
                        max_return: journal.max_return,
                    },
                );
        }

        fn update_verifier_address(
            ref self: ContractState, new_verifier_address: starknet::ContractAddress,
        ) {
            self.ownable.assert_only_owner();
            self
                .bn254_verifier
                .write(
                    IRisc0Groth16VerifierBN254Dispatcher { contract_address: new_verifier_address },
                );
        }

        fn get_groth16_verifier_address(self: @ContractState) -> starknet::ContractAddress {
            self.bn254_verifier.read().contract_address
        }

        fn get_pitchlake_client_address(self: @ContractState) -> starknet::ContractAddress {
            self.pitchlake_client.read().contract_address
        }

        fn upgrade(ref self: ContractState, new_class_hash: starknet::ClassHash) {
            self.ownable.assert_only_owner();
            self.upgradeable.upgrade(new_class_hash);
        }
    }
}
