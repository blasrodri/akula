// use bytes::Bytes;
// use ethereum_types::Address;

// #[async_trait]
// pub trait StateBuffer {
//     // Readers

//     async fn read_account(&self, address: Address) -> Option<Account>;

//     async fn read_code(&self, code_hash: H256) -> Bytes;

//     async fn read_storage(&self, address: Address, incarnation: u64, location: H256) -> H256;

//     // Previous non-zero incarnation of an account; 0 if none exists.
//     async fn previous_incarnation(&self, address: Address) -> u64;

//     async fn read_header(&self, block_number: u64, block_hash: H256) -> Option<BlockHeader>;

//     async fn read_body(&self, block_number: u64, block_hash: H256) -> Option<BlockBody>;

//   virtual std::optional<intx::uint256> total_difficulty(uint64_t block_number,
//                                                         const evmc::bytes32& block_hash) const noexcept = 0;

//   virtual evmc::bytes32 state_root_hash() const = 0;

//   virtual uint64_t current_canonical_block() const = 0;

//   virtual std::optional<evmc::bytes32> canonical_hash(uint64_t block_number) const = 0;

//   ///@}

//   virtual void insert_block(const Block& block, const evmc::bytes32& hash) = 0;

//   virtual void canonize_block(uint64_t block_number, const evmc::bytes32& block_hash) = 0;

//   virtual void decanonize_block(uint64_t block_number) = 0;

//   virtual void insert_receipts(uint64_t block_number, const std::vector<Receipt>& receipts) = 0;

//   /** @name State changes
//    *  Change sets are backward changes of the state, i.e. account/storage values <em>at the beginning of a block</em>.
//    */
//   ///@{

//   /** Mark the beggining of a new block.
//    * Must be called prior to calling update_account/update_account_code/update_storage.
//    */
//   virtual void begin_block(uint64_t block_number) = 0;

//   virtual void update_account(const evmc::address& address, std::optional<Account> initial,
//                               std::optional<Account> current) = 0;

//   virtual void update_account_code(const evmc::address& address, uint64_t incarnation, const evmc::bytes32& code_hash,
//                                    ByteView code) = 0;

//   virtual void update_storage(const evmc::address& address, uint64_t incarnation, const evmc::bytes32& location,
//                               const evmc::bytes32& initial, const evmc::bytes32& current) = 0;

//   virtual void unwind_state_changes(uint64_t block_number) = 0;
// }
