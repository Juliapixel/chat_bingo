echo "Deploying static Blazor WASM website and NGINX config..."
echo "Files will deploy as such:"
echo "/webstaging/* -> /var/wwww/web/*"
echo "./nginx_blazor.conf -> /etc/nginx/nginx.conf"
echo "./mime_blazor.types -> /etc/nginx/mime.types"
read -p "Press enter to continue"

echo Stopping NGINX...
sudo nginx -s quit

echo Copying config files...
sudo cp ./nginx_blazor.conf /etc/nginx/nginx.conf
sudo cp ./mime_blazor.types /etc/nginx/mime.types

echo Copying Blazor files...
sudo cp webstaging/* /var/www/web -r

echo Restarting NGINX...
sudo nginx

echo "Sync Complete, remember to purge CloudFlare cache if changes aren't reflected"
