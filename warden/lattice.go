package main

import "fmt"

// MotionLattice enforces deterministic state transitions for requests.
type MotionLattice struct {
	allowed map[string][]string
}

// NewMotionLattice builds a default lattice with validation and action states.
func NewMotionLattice() *MotionLattice {
	return &MotionLattice{
		allowed: map[string][]string{
			"ingress":   {"validate"},
			"validate":  {"authorize"},
			"authorize": {"route", "reject"},
			"route":     {"log", "complete"},
			"reject":    {"log"},
			"log":       {"complete"},
		},
	}
}

// ValidateTransition returns error when a transition is not permitted.
func (m *MotionLattice) ValidateTransition(current, next string) error {
	allowedNext, ok := m.allowed[current]
	if !ok {
		return fmt.Errorf("unknown state %s", current)
	}
	for _, candidate := range allowedNext {
		if candidate == next {
			return nil
		}
	}
	return fmt.Errorf("invalid transition %s -> %s", current, next)
}
