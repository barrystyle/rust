

async fn handle_new_tx(this: Arc<MsgHandler>, peer: Option<Arc<Peer>>, body: Vec<u8>) {
    // println!("1111111 handle_txblock_arrive Tx, peer={} len={}", peer.nick(), body.clone().len());
    // parse
    let txpkg = transaction::create_pkg(BytesW4::from_vec(body));
    if let Err(e) = txpkg {
        return // parse tx error
    }
    let txpkg = txpkg.unwrap();
    // tx hash with fee
    let hxfe = txpkg.objc().hash_with_fee();
    let (already, knowkey) = check_know(&this.knows, &hxfe, peer.clone());
    if already {
        return  // alreay know it
    }
    let txdatas = txpkg.body().clone().into_vec();
    let is_open_miner = this.engine.config().miner_enable;
    if is_open_miner {
        // try execute tx
        if let Err(..) = this.engine.try_execute_tx(txpkg.objc().as_ref().as_read()) {
            return // tx execute fail
        }
        // add to pool
        this.txpool.insert(txpkg);
    }
    // broadcast
    let p2p = this.p2pmng.lock().unwrap();
    let p2p = p2p.as_ref().unwrap();
    p2p.broadcast_message(1/*delay*/, knowkey, MSG_TX_SUBMIT, txdatas);
}


async fn handle_new_block(this: Arc<MsgHandler>, peer: Option<Arc<Peer>>, body: Vec<u8>) {
    // println!("222222222222 handle_txblock_arrive Block len={}",  body.clone().len());
    let mut blkhead = BlockIntro::default();
    if let Err(_) = blkhead.parse(&body, 0) {
        return // parse tx error
    }
    let blkhei = blkhead.height().uint();
    let blkhx = blkhead.hash();
    let (already, knowkey) = check_know(&this.knows, &blkhx, peer.clone());
    if already {
        return  // alreay know it
    }
    // check height and difficulty (mint consensus)
    let eng = this.engine.clone();
    let engcnf = eng.config();
    let is_open_miner = engcnf.miner_enable;
    let heispan = engcnf.unstable_block;
    let latest = eng.latest_block();
    let lathei = latest.objc().height().uint();
    if blkhei > heispan && blkhei < lathei - heispan {
        return // height too late
    }
    let mintckr = eng.mint_checker();
    let stoptr = eng.store();
    // may insert
    if blkhei <= lathei + 1 {
        // prepare check
        if let Err(_) = mintckr.prepare(stoptr.as_ref(), &blkhead) {
            return  // difficulty check fail
        }
        // do insert  ◆ ◇ ⊙ ■ □ △ ▽ ❏ ❐ ❑ ❒  ▐ ░ ▒ ▓ ▔ ▕ ■ □ ▢ ▣ ▤ ▥ ▦ ▧ ▨ ▩ ▪ ▫    
        let hxtail = &blkhx.as_bytes()[24..];
        let txs = blkhead.transaction_count().uint() - 1;
        let blkts = &timeshow(blkhead.timestamp().uint())[14..];
        print!("❏ discover block {} …{} txs{:2} time {} inserting at {} ... ", 
            blkhei, hex::encode(hxtail), txs, blkts, &ctshow()[11..]);
        let bodycp = body.clone();
        let engptr = eng.clone();
        let txpool = this.txpool.clone();
        std::thread::spawn(move||{
            // create block
            let blkpkg = block::create_pkg(BytesW4::from_vec(bodycp));
            if let Err(e) = blkpkg {
                return // parse error
            }
            let blkp = blkpkg.unwrap();
            let thsx = blkp.objc().transaction_hash_list(false); // hash no fee
            if let Err(e) = engptr.insert(blkp) {
                println!("Error: {}", e);
            }else{
                println!("ok.");
                if is_open_miner {
                    drain_all_block_txs(engptr.state().clone(), txpool, thsx, blkhei);
                }
            }
        });
    }else{
        // req sync
        if let Some(ref pr) = peer {
            send_req_block_hash_msg(pr.clone(), (heispan+1) as u8, lathei).await;
        }
        return // not broadcast
    }
    // broadcast new block
    let p2p = this.p2pmng.lock().unwrap();
    let p2p = p2p.as_ref().unwrap();
    p2p.broadcast_message(0/*not delay*/, knowkey, MSG_BLOCK_DISCOVER, body);
}



// return already know
fn check_know(mine: &Knowledge, hxkey: &Hash, peer: Option<Arc<Peer>>) -> (bool, KnowKey) {
    let knowkey: [u8; KNOWLEDGE_SIZE] = hxkey.clone().into_array();
    if let Some(ref pr) = peer {
        pr.knows.add(knowkey.clone());
    }
    if mine.check(&knowkey) {
        return (true, knowkey) // alreay know it
    }
    mine.add(knowkey.clone());
    (false, knowkey)
}


// drain_all_block_txs
fn drain_all_block_txs(sta: Arc<dyn State>, txpool: Arc<dyn TxPool>, txs: Vec<Hash>, blkhei: u64) {
    if blkhei % 15 == 0 {
        println!("{}.", txpool.print());
    }
    // drop all overdue diamond mint tx
    if blkhei % 5 == 0 {
        let ldn = MintStateDisk::wrap(sta.as_ref()).latest_diamond().number.uint();
        txpool.drain_filter_at(&|a: &Box<dyn TxPkg>| {
            let tx = a.objc().as_read();
            let dn = get_diamond_mint_number(tx);
            // println!("TXPOOL: drain_filter_at dmint, tx: {}, dn: {}, last dn: {}", tx.hash().hex(), dn, ldn);
            dn + 1 != ldn // is not next
        }, TXPOOL_GROUP_DIAMOND_MINT);
    }
    // drop all exist normal tx
    txpool.drain(&txs);
}


// for diamond create action
fn get_diamond_mint_number(tx: &dyn TransactionRead) -> u32 {
    const DMINT: u16 = mint_action::ACTION_KIND_ID_DIAMOND_MINT;
    let mut num: u32 = 0;
    for act in tx.actions() {
        if act.kind() == DMINT {
            let dm = mint_action::DiamondMint::must(&act.serialize());
            num = dm.head.number.uint();
            break;
        }
    }
    num
} 