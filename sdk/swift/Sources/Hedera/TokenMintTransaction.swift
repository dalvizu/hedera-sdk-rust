import Foundation

/// Mint tokens to the token's treasury account.
public final class TokenMintTransaction: Transaction {
    /// Create a new `TokenMintTransaction`.
    public init(
        tokenId: TokenId? = nil,
        amount: UInt64 = 0,
        metadata: [Data] = []
    ) {
        self.tokenId = tokenId
        self.amount = amount
        self.metadata = metadata
    }

    /// The token for which to mint tokens.
    public var tokenId: TokenId?

    /// Sets the token for which to mint tokens.
    @discardableResult
    public func tokenId(_ tokenId: TokenId) -> Self {
        self.tokenId = tokenId

        return self
    }

    /// The amount of a fungible token to mint to the treasury account.
    public var amount: UInt64

    //// Sets the amount of a fungible token to mint to the treasury account.
    @discardableResult
    public func amount(_ amount: UInt64) -> Self {
        self.amount = amount

        return self
    }

    /// The list of metadata for a non-fungible token to mint to the treasury account.
    public var metadata: [Data]

    /// Sets the list of metadata for a non-fungible token to mint to the treasury account.
    @discardableResult
    public func metadata(_ metadata: [Data]) -> Self {
        self.metadata = metadata

        return self
    }

    private enum CodingKeys: String, CodingKey {
        case tokenId
        case amount
        case metadata
    }

    public override func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        try container.encode(tokenId, forKey: .tokenId)
        try container.encode(amount, forKey: .amount)
        try container.encode(metadata.map { $0.base64EncodedString() }, forKey: .metadata)

        try super.encode(to: encoder)
    }
}
