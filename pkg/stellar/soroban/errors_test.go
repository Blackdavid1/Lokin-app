package soroban

import (
	"errors"
	"testing"

	"github.com/blurbeast/loki-app/pkg/apperrors"
)

func TestClassifySorobanError_Circle(t *testing.T) {
	tests := []struct {
		name     string
		code     uint32
		expected error
	}{
		{"Circle Not Initialized", 1, apperrors.ErrCircleNotInitialized},
		{"Circle Not Active", 2, apperrors.ErrCircleNotActive},
		{"Circle Full", 3, apperrors.ErrCircleFull},
		{"Already Member", 4, apperrors.ErrAlreadyMember},
		{"Not Member", 5, apperrors.ErrNotMember},
		{"Moi Score Too Low", 6, apperrors.ErrMoiScoreTooLow},
		{"Round Not Current", 7, apperrors.ErrRoundNotCurrent},
		{"Invalid Amount", 8, apperrors.ErrInvalidAmount},
		{"Payment Deadline Passed", 9, apperrors.ErrPaymentDeadlinePassed},
		{"Max Strikes Reached", 10, apperrors.ErrMaxStrikesReached},
		{"Not Organizer", 11, apperrors.ErrNotOrganizer},
		{"Contract Paused", 12, apperrors.ErrContractPaused},
		{"Invalid Invite Code", 13, apperrors.ErrInvalidInviteCode},
		{"Auction Already Resolved", 14, apperrors.ErrAuctionAlreadyResolved},
		{"Vote Quorum Not Met", 15, apperrors.ErrVoteQuorumNotMet},
		{"Already Contributed", 16, apperrors.ErrAlreadyContributed},
		{"Already Voted", 17, apperrors.ErrAlreadyVoted},
		{"Already Bidded", 18, apperrors.ErrAlreadyBidded},
		{"Payout Already Executed", 19, apperrors.ErrPayoutAlreadyExecuted},
		{"Invalid Payout Type", 20, apperrors.ErrInvalidPayoutType},
		{"Invalid Round", 21, apperrors.ErrInvalidRound},
		{"Contribution Mismatch", 22, apperrors.ErrContributionMismatch},
		{"Circle Not Full", 23, apperrors.ErrCircleNotFull},
		{"Not Enough Votes", 24, apperrors.ErrNotEnoughVotes},
		{"Dispute Already Raised", 25, apperrors.ErrDisputeAlreadyRaised},
		{"No Active Dispute", 26, apperrors.ErrNoActiveDispute},
		{"Unauthorized", 27, apperrors.ErrUnauthorized},
		{"Invalid Bid", 28, apperrors.ErrInvalidBid},
		{"Invalid Member Status", 29, apperrors.ErrInvalidMemberStatus},
		{"Empty Payout Order", 30, apperrors.ErrEmptyPayoutOrder},
		{"Circle Size Exceeds Tier", 31, apperrors.ErrCircleSizeExceedsTier},
		{"Contribution Exceeds Tier", 32, apperrors.ErrContributionExceedsTier},
		{"Vector Access Error", 33, apperrors.ErrVecAccessError},
		{"Allowlist Not Permitted", 34, apperrors.ErrAllowlistNotPermitted},
		{"Insufficient Contract Balance", 35, apperrors.ErrInsufficientContractBalance},
		{"Self Referral", 36, apperrors.ErrSelfReferral},
		{"Oracle Unavailable", 37, apperrors.ErrOracleUnavailable},
		{"Payout Already Scheduled", 59, apperrors.ErrPayoutAlreadyScheduled},
		{"Dutch Auction Not Configured", 60, apperrors.ErrDutchAuctionNotConfigured},
		{"Dutch Auction Expired", 61, apperrors.ErrDutchAuctionExpired},
		{"Invalid Dutch Config", 62, apperrors.ErrInvalidDutchConfig},
		{"Auction Not Resolved", 63, apperrors.ErrAuctionNotResolved},
		{"Already Struck", 67, apperrors.ErrAlreadyStruck},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			scErr := &SorobanContractError{
				Contract: ContractTypeCircle,
				Code:     tt.code,
				Message:  tt.name,
			}
			err := ClassifySorobanError(scErr)
			if !errors.Is(err, tt.expected) {
				t.Errorf("ClassifySorobanError() code %d: got %v, want %v", tt.code, err, tt.expected)
			}
		})
	}
}

func TestClassifySorobanError_Factory(t *testing.T) {
	tests := []struct {
		code     uint32
		expected error
	}{
		{1, apperrors.ErrFactoryNotInitialized},
		{2, apperrors.ErrUnauthorized},
		{3, apperrors.ErrContractPaused},
		{4, apperrors.ErrWasmHashNotSet},
		{5, apperrors.ErrInvalidFeeBps},
		{6, apperrors.ErrCircleDeployFailed},
		{7, apperrors.ErrInvalidConfig},
		{8, apperrors.ErrRateLimitExceeded},
	}

	for _, tt := range tests {
		scErr := &SorobanContractError{
			Contract: ContractTypeCircleFactory,
			Code:     tt.code,
		}
		err := ClassifySorobanError(scErr)
		if !errors.Is(err, tt.expected) {
			t.Errorf("Factory code %d: got %v, want %v", tt.code, err, tt.expected)
		}
	}
}

func TestClassifySorobanError_Treasury(t *testing.T) {
	tests := []struct {
		code     uint32
		expected error
	}{
		{1, apperrors.ErrTreasuryNotInitialized},
		{2, apperrors.ErrUnauthorized},
		{3, apperrors.ErrContractPaused},
		{4, apperrors.ErrInsufficientBalance},
		{5, apperrors.ErrInvalidAmount},
		{6, apperrors.ErrAlreadyInitialized},
		{7, apperrors.ErrContractNotPaused},
	}

	for _, tt := range tests {
		scErr := &SorobanContractError{
			Contract: ContractTypeTreasury,
			Code:     tt.code,
		}
		err := ClassifySorobanError(scErr)
		if !errors.Is(err, tt.expected) {
			t.Errorf("Treasury code %d: got %v, want %v", tt.code, err, tt.expected)
		}
	}
}

func TestClassifySorobanError_Nil(t *testing.T) {
	if err := ClassifySorobanError(nil); err != nil {
		t.Errorf("expected nil for nil input, got %v", err)
	}
}

func TestClassifySorobanError_UnknownCode(t *testing.T) {
	scErr := &SorobanContractError{
		Contract: ContractTypeCircle,
		Code:     9999,
		Message:  "unknown code",
	}
	err := ClassifySorobanError(scErr)
	if err == nil {
		t.Fatal("expected error for unknown code, got nil")
	}
	if err.Error() != "circle error 9999: unknown code" {
		t.Errorf("unexpected error format: %s", err.Error())
	}
}
