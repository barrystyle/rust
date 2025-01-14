
pub const ACTION_KIND_ID_DIAMOND_MINT: u16 = 4;

/**
 * reg actions
 */
 pubFnRegExtendActionCreates!{

    ChannelOpen      // 2
    ChannelClose     // 3

    DiamondMint              // 4
    DiamondTransfer          // 5 
    DiamondFromToTransfer    // 6
    DiamondMultipleTransfer  // 7

    DiamondInscription       // 32
    DiamondInscriptionClean  // 33

}

// reg action
pub fn init_reg() {
    unsafe {
        crate::protocol::action::EXTEND_ACTIONS_TRY_CREATE_FUNC = Some(try_create);   
    }
}