docker image build -t 403p1 .
docker run --net cluster --ip 172.18.0.2 403p1 &
docker run --net cluster --ip 172.18.0.3 403p1 &
docker run --net cluster --ip 172.18.0.4 403p1 &
docker run --net cluster --ip 172.18.0.5 403p1 &
docker run --net cluster --ip 172.18.0.6 403p1 &
echo
echo
docker ps -a
