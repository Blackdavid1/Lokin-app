package soroban

import (
	"fmt"

	"github.com/blurbeast/loki-app/pkg/apperrors"
)

// ContractType identifies the origin contract of a Soroban error.
type ContractType string

const (
	ContractTypeCircle             ContractType = "circle"
	ContractTypeCircleFactory      ContractType = "circle-factory"
	ContractTypeTreasury           ContractType = "treasury"
	ContractTypeReputationRegistry ContractType = "reputation-registry"
	ContractTypeStaking            ContractType = "staking"
	ContractTypeGovernance         ContractType = "governance"
	ContractTypeGovernanceToken    ContractType = "governance-token"
	ContractTypeEscrowSwap         ContractType = "escrow-swap"
)

// SorobanContractError represents a typed error returned by a Soroban contract execution.
type SorobanContractError struct {
	Contract ContractType `json:"contract"`
	Code     uint32       `json:"code"`
	Message  string       `json:"message"`
}

func (e *SorobanContractError) Error() string {
	if e.Contract != "" {
		return fmt.Sprintf("%s contract error %d: %s", e.Contract, e.Code, e.Message)
	}
	return fmt.Sprintf("contract error %d: %s", e.Code, e.Message)
}

// ClassifySorobanError maps Soroban contract numeric error codes to Go domain errors.
func ClassifySorobanError(scErr *SorobanContractError) error {
	if scErr == nil {
		return nil
	}

	switch scErr.Contract {
	case ContractTypeCircleFactory:
		return classifyFactoryError(scErr.Code, scErr.Message)
	case ContractTypeTreasury:
		return classifyTreasuryError(scErr.Code, scErr.Message)
	case ContractTypeReputationRegistry:
		return classifyReputationError(scErr.Code, scErr.Message)
	case ContractTypeStaking:
		return classifyStakingError(scErr.Code, scErr.Message)
	case ContractTypeGovernance:
		return classifyGovernanceError(scErr.Code, scErr.Message)
	default:
		// Default to Circle contract error mapping (primary contract)
		return classifyCircleError(scErr.Code, scErr.Message)
	}
}

// ClassifyCircleError explicitly maps circle contract numeric error codes.
func ClassifyCircleError(code uint32) error {
	return classifyCircleError(code, "")
}

func classifyCircleError(code uint32, msg string) error {
	switch code {
	case 1:
		return apperrors.ErrCircleNotInitialized
	case 2:
		return apperrors.ErrCircleNotActive
	case 3:
		return apperrors.ErrCircleFull
	case 4:
		return apperrors.ErrAlreadyMember
	case 5:
		return apperrors.ErrNotMember
	case 6:
		return apperrors.ErrMoiScoreTooLow
	case 7:
		return apperrors.ErrRoundNotCurrent
	case 8:
		return apperrors.ErrInvalidAmount
	case 9:
		return apperrors.ErrPaymentDeadlinePassed
	case 10:
		return apperrors.ErrMaxStrikesReached
	case 11:
		return apperrors.ErrNotOrganizer
	case 12:
		return apperrors.ErrContractPaused
	case 13:
		return apperrors.ErrInvalidInviteCode
	case 14:
		return apperrors.ErrAuctionAlreadyResolved
	case 15:
		return apperrors.ErrVoteQuorumNotMet
	case 16:
		return apperrors.ErrAlreadyContributed
	case 17:
		return apperrors.ErrAlreadyVoted
	case 18:
		return apperrors.ErrAlreadyBidded
	case 19:
		return apperrors.ErrPayoutAlreadyExecuted
	case 20:
		return apperrors.ErrInvalidPayoutType
	case 21:
		return apperrors.ErrInvalidRound
	case 22:
		return apperrors.ErrContributionMismatch
	case 23:
		return apperrors.ErrCircleNotFull
	case 24:
		return apperrors.ErrNotEnoughVotes
	case 25:
		return apperrors.ErrDisputeAlreadyRaised
	case 26:
		return apperrors.ErrNoActiveDispute
	case 27:
		return apperrors.ErrUnauthorized
	case 28:
		return apperrors.ErrInvalidBid
	case 29:
		return apperrors.ErrInvalidMemberStatus
	case 30:
		return apperrors.ErrEmptyPayoutOrder
	case 31:
		return apperrors.ErrCircleSizeExceedsTier
	case 32:
		return apperrors.ErrContributionExceedsTier
	case 33:
		return apperrors.ErrVecAccessError
	case 34:
		return apperrors.ErrAllowlistNotPermitted
	case 35:
		return apperrors.ErrInsufficientContractBalance
	case 36:
		return apperrors.ErrSelfReferral
	case 37:
		return apperrors.ErrOracleUnavailable
	case 59:
		return apperrors.ErrPayoutAlreadyScheduled
	case 60:
		return apperrors.ErrDutchAuctionNotConfigured
	case 61:
		return apperrors.ErrDutchAuctionExpired
	case 62:
		return apperrors.ErrInvalidDutchConfig
	case 63:
		return apperrors.ErrAuctionNotResolved
	case 67:
		return apperrors.ErrAlreadyStruck
	default:
		if msg != "" {
			return fmt.Errorf("circle error %d: %s", code, msg)
		}
		return fmt.Errorf("circle error %d", code)
	}
}

func classifyFactoryError(code uint32, msg string) error {
	switch code {
	case 1:
		return apperrors.ErrFactoryNotInitialized
	case 2:
		return apperrors.ErrUnauthorized
	case 3:
		return apperrors.ErrContractPaused
	case 4:
		return apperrors.ErrWasmHashNotSet
	case 5:
		return apperrors.ErrInvalidFeeBps
	case 6:
		return apperrors.ErrCircleDeployFailed
	case 7:
		return apperrors.ErrInvalidConfig
	case 8:
		return apperrors.ErrRateLimitExceeded
	default:
		if msg != "" {
			return fmt.Errorf("circle-factory error %d: %s", code, msg)
		}
		return fmt.Errorf("circle-factory error %d", code)
	}
}

func classifyTreasuryError(code uint32, msg string) error {
	switch code {
	case 1:
		return apperrors.ErrTreasuryNotInitialized
	case 2:
		return apperrors.ErrUnauthorized
	case 3:
		return apperrors.ErrContractPaused
	case 4:
		return apperrors.ErrInsufficientBalance
	case 5:
		return apperrors.ErrInvalidAmount
	case 6:
		return apperrors.ErrAlreadyInitialized
	case 7:
		return apperrors.ErrContractNotPaused
	default:
		if msg != "" {
			return fmt.Errorf("treasury error %d: %s", code, msg)
		}
		return fmt.Errorf("treasury error %d", code)
	}
}

func classifyReputationError(code uint32, msg string) error {
	switch code {
	case 1:
		return apperrors.ErrCircleNotInitialized
	case 2:
		return apperrors.ErrUnauthorized
	case 3:
		return apperrors.ErrContractPaused
	case 4:
		return apperrors.ErrInvalidActivityType
	case 5:
		return apperrors.ErrScoreNotFound
	case 6:
		return apperrors.ErrInvalidScoreImpact
	case 7:
		return apperrors.ErrScoreOverflow
	default:
		if msg != "" {
			return fmt.Errorf("reputation-registry error %d: %s", code, msg)
		}
		return fmt.Errorf("reputation-registry error %d", code)
	}
}

func classifyStakingError(code uint32, msg string) error {
	switch code {
	case 1:
		return apperrors.ErrAlreadyStaked
	case 2:
		return apperrors.ErrNoActiveStake
	case 3:
		return apperrors.ErrInvalidPeriod
	case 4:
		return apperrors.ErrInvalidAmount
	case 5:
		return apperrors.ErrLockPeriodNotEnded
	case 6:
		return apperrors.ErrNoUnbondingPosition
	case 7:
		return apperrors.ErrUnbondingNotEnded
	case 8:
		return apperrors.ErrInsufficientBalance
	case 9:
		return apperrors.ErrUnauthorized
	case 10:
		return apperrors.ErrContractPaused
	case 11:
		return apperrors.ErrAmountBelowMinimum
	default:
		if msg != "" {
			return fmt.Errorf("staking error %d: %s", code, msg)
		}
		return fmt.Errorf("staking error %d", code)
	}
}

func classifyGovernanceError(code uint32, msg string) error {
	switch code {
	case 1:
		return apperrors.ErrProposalNotFound
	case 2:
		return apperrors.ErrProposalNotActive
	case 3:
		return apperrors.ErrAlreadyVotedGov
	case 4:
		return apperrors.ErrVotingClosed
	case 5:
		return apperrors.ErrTimelockNotMet
	case 6:
		return apperrors.ErrProposalPassed
	case 7:
		return apperrors.ErrProposalFailed
	case 8:
		return apperrors.ErrUnauthorized
	case 9:
		return apperrors.ErrContractPaused
	default:
		if msg != "" {
			return fmt.Errorf("governance error %d: %s", code, msg)
		}
		return fmt.Errorf("governance error %d", code)
	}
}
