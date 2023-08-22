

header_type ethernet_t {
	fields {
		dst_addr : 48;
		src_addr : 48;
		etherType : 16;
	}
}

header_type ipv4_t {
	fields {
		version : 4;
		ihl : 4;
		diffserv : 8;
		totalLen : 16;
		identification : 16;
		flags : 3;
		fragOffset : 13;
		ttl : 8;
		protocol : 8;
		hdrChecksum : 16;
		src_addr : 32;
		dst_addr: 32;
	}
} 

header_type tcp_t {
	fields {
		srcPort : 16;
		dstPort : 16;
		seq_no : 32;
		ack_no : 32;
		dataOffset : 4;
        res : 6;
		flags : 6;	 
        window : 16;
        checksum : 16;
        urgentPtr : 16;
    }
}

// header cpu_header_t cpu_header;
header ethernet_t ethernet;
header ipv4_t ipv4;
header tcp_t tcp;
//********HEADERS END********
