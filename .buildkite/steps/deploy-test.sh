#!/bin/bash
set -euo pipefail

while getopts t: option; do
case "${option}" in
    t) IMAGETAG=${OPTARG};;
    *) echo "Usage: $0 [OPTIONS]. Where OPTIONS can be:"
       echo "    -t <IMAGETAG>  tag for cybercoredev/evm_loader Docker-image"
       exit 1;;
esac
done

REVISION=$(git rev-parse HEAD)
EVM_LOADER_IMAGE=cybercoredev/evm_loader:${IMAGETAG:-$REVISION}

docker-compose -f evm_loader/docker-compose-test.yml up -d

function cleanup_docker {
    docker logs solana >solana.log 2>&1
    echo "Cleanup docker-compose..."
    docker-compose -f evm_loader/docker-compose-test.yml down
    echo "Cleanup docker-compose done."
}
trap cleanup_docker EXIT
sleep 10

echo "Run tests..."
#docker run --rm --network evm_loader-deploy_test-net -ti \
#     -e SOLANA_URL=http://solana:8899 \
#     --workdir /opt/ \
#     ${EXTRA_ARGS:-} \
#     $EVM_LOADER_IMAGE 'deploy-test.sh'
echo "Run tests return"

echo "Run ERC20 tests..."
docker run --rm --network evm_loader-deploy_test-net -ti \
     -e SOLANA_URL=http://solana:8899 \
     --workdir /opt/ERC20/ \
     ${EXTRA_ARGS:-} \
     $EVM_LOADER_IMAGE 'deploy-test.sh'
echo "Run ERC20 tests return"

exit $?
