


pub struct BlockMintChecker {

}

impl BlockMintChecker {
    pub fn create() -> BlockMintChecker {
        BlockMintChecker{}
    }
}


impl MintChecker for BlockMintChecker {

    fn consensus(&self, cbtx: &dyn Block) -> RetErr {
        impl_consensus(self, cbtx)
    }

    fn coinbase(&self, height: u64, cbtx: &dyn Transaction) -> RetErr {
        impl_coinbase(self, height, cbtx)
    }

    fn initialize(&self, state: &mut dyn StoreDB) -> RetErr {
        impl_initialize(self, state)
    } 

    fn genesis(&self) -> Box<dyn BlockPkg> {
        let gnsblk = create_genesis_block();
        // let body = gnsblk.serialize();
        let pkg = BlockPackage::new(Box::new(gnsblk));
        Box::new(pkg)
    }
}