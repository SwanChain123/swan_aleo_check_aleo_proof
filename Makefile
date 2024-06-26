VERSION=0.0.1
NAME=swan-check-aleo-proof
REPO=martian123

.PHONY: build
build: 
	cargo build --release && \
	cp ./target/release/swan_aleo_check_aleo_proof ./swan-check-aleo-proof

.PHONY: run
run: build
	 ./swan-check-aleo-proof -p 3031

.PHONY: docker
docker: build
	@sudo docker build -t $(REPO)/$(NAME):$(VERSION) . 

.PHONY: push
push: docker
	@sudo docker push $(REPO)/$(NAME):$(VERSION)

.PHONY: test_T
test_T: 
	curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"check_aleo_proof","params":{"input_param":"{\"address\":\"aleo1n6utz6c8gtgqp4xfkm03ckf4n9fupuv3jf0feea9ggt547skasxq423ajy\",\"nonce_ex\":122,\"nonce_len\":1,\"min_proof_target\":600}", "proof":"task_id:10001,nonce:8837783779460058597,challenge:f1140000cd0816c483694645784e4024bdb528620b73ef5b6658a5ee004dfd9319985404ff1f0000,solution:puzzle1y2xn62m6ahr3kyqdsnv043xlt8993hdwse87ak9zthgq6rmecnrdh2qfdz36527ur8vz3m9xdahcqxmcq5q,proof:7e4bd4882b45a5e0407ad732aeab9072865b52853005c94b8e0e098f10c07087c8ac5d5514ed13ded7f689ca5f67310000,target:602"},"id":1}' http://127.0.0.1:8010 | jq


.PHONY: test_F
test_F: 
	curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"check_aleo_proof","params":{"input_param":"{\"address\":\"aleo1n6utz6c8gtgqp4xfkm03ckf4n9fupuv3jf0feea9ggt547skasxq423ajy\",\"nonce_ex\":123,\"nonce_len\":1,\"min_proof_target\":1000}", "proof":"task_id:10001,nonce:8837783779460058597,challenge:f1140000cd0816c483694645784e4024bdb528620b73ef5b6658a5ee004dfd9319985404ff1f0000,solution:puzzle1y2xn62m6ahr3kyqdsnv043xlt8993hdwse87ak9zthgq6rmecnrdh2qfdz36527ur8vz3m9xdahcqxmcq5q,proof:7e4bd4882b45a5e0407ad732aeab9072865b52853005c94b8e0e098f10c07087c8ac5d5514ed13ded7f689ca5f67310000,target:602"},"id":101}' http://127.0.0.1:8010 | jq
