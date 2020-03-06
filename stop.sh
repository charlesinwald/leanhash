PID=$(/usr/sbin/lsof -ti:34254)
sudo kill $PID
