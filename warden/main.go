package main

import (
	"log"
	"net"

	"google.golang.org/grpc"
	"google.golang.org/grpc/reflection"
)

func main() {
	lattice := NewMotionLattice()

	server := grpc.NewServer(
		grpc.UnaryInterceptor(unarySecurityInterceptor(lattice)),
		grpc.StreamInterceptor(streamSecurityInterceptor(lattice)),
	)

	// Reflection is enabled to support debugging clients; remove if not needed.
	reflection.Register(server)

	lis, err := net.Listen("tcp", ":50051")
	if err != nil {
		log.Fatalf("failed to listen: %v", err)
	}

	log.Println("AxiomHive Warden listening on :50051 with deterministic interceptors")
	if err := server.Serve(lis); err != nil {
		log.Fatalf("failed to serve: %v", err)
	}
}
