ssh -t -i ~/Desktop/403Cluster.pem ec2-user@ec2-3-17-221-247.us-east-2.compute.amazonaws.com 'cd 403p1/403p1; nohup ./start.sh > server_output 2>  server_err < /dev/null' 
ssh -t -i ~/Desktop/403Cluster.pem ec2-user@ec2-13-58-219-209.us-east-2.compute.amazonaws.com 'cd 403p1/; nohup ./start.sh > server_output 2> server_err < /dev/null'
ssh -t -i ~/Desktop/403Cluster.pem ec2-user@ec2-3-12-195-112.us-east-2.compute.amazonaws.com 'cd 403p1/; nohup ./start.sh > server_output 2> server_err < /dev/null' 

