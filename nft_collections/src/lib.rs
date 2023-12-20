//stuff to add
//not only xrd, but other crypots aswell to be more general
//delte the first inital supply, just need a way to create a new vault withou the bucket fucntion, cant figure out new beacuse need a resource address

use scrypto::prelude::*;

#[derive(ScryptoSbor)]
pub enum Rarity {
    Common,
    Rare,
    MythicRare,
}

#[derive(NonFungibleData, ScryptoSbor)]
pub struct Nft {
    key_image_url: Url,
    rarity: Rarity,
    level: u8,
}

#[blueprint]
mod nfts {

    struct NftCollection {
        nfts: NonFungibleVault, //a vault that holds all the nfts
        nft_price: Decimal, // the price for an nft
        nft_manager: ResourceManager,//the resource address off all nfts
        nft_id_counter: u64, // A counter for ID generationcoun
        collected_crypto: FungibleVault, //A vault that collects all xrd payments
        number_of_nfts: u32,
        admin_badge: ResourceAddress,
    }

    impl NftCollection {
        pub fn instantiate_component(name: String, price: Decimal, number_of_nfts: u32) -> (Global<NftCollection>, FungibleBucket) {

            let admin_badge = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata!(
                    init {
                        "name" => "nftAdminBadge", locked;
                    }))
                .mint_initial_supply(1);

            let nft =
                ResourceBuilder::new_integer_non_fungible::<Nft>(OwnerRole::None)
                .metadata(metadata!(
                    init {
                        "name" => name, locked;
                        // "description" => description, locked; //string
                        // "tags" => collection_name, ; //vec<string>
                        // "icon_url" => icon_url; //url
                        // "info_url" => info_url; //url
                        // "royalities" => royalities; //what is the standard, just as metadata, or withdraw rules?
                    }
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(allow_all); //require admin badge on this for later
                    minter_updater => rule!(allow_all); //require admin badge on this for later
                ))
                .create_with_no_initial_supply();


            // Instantiate our component
            let component = Self {
                nfts: scrypto::prelude::NonFungibleVault(nft.create_empty_vault()),
                nft_price: price,
                nft_manager: nft,
                nft_id_counter: 0,
                collected_crypto: FungibleVault::new(XRD),
                number_of_nfts: number_of_nfts,
                admin_badge: admin_badge.resource_address(), 
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();
            (component, admin_badge)
        }



        //buy from the component, make this random and so you cant se output
        pub fn buy_nft(
            &mut self,
            key: u64,
            mut payment: FungibleBucket,
        ) -> (NonFungibleBucket, FungibleBucket) {

            //let remaining_nfts=self.nfts.non_fungible_local_ids(self.number_of_nfts); //get all ids remaining in the vault
            //let key=take a random of these nfts

            self.collected_crypto.put(payment.take(self.nft_price)); // get paid

            // Take the requested NFT
            let id: NonFungibleLocalId = key.into();
            let nft = self.nfts.take_non_fungible(&id);

            // Return the NFT and change
            (nft, payment)
        }

        //make input parameteers here for metadata
        pub fn mint_nft(&mut self){

            let nft_bucket = self.nft_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(self.nft_id_counter),
                Nft {
                    rarity: Rarity::Common,
                    level: 1,
                    key_image_url: Url::of("https://pyro-public.s3.eu-central-1.amazonaws.com/collections/1/JPG_640px/Pyro_2.jpg")
                },
            ).as_non_fungible();

            self.nfts.put(nft_bucket);//puts into the nft vault

            self.nft_id_counter += 1;
            if self.nft_id_counter >= self.number_of_nfts.into(){ //only x is mintable 0,...,x-1
                self.nft_manager.lock_mintable();
            }
        }
    }
}