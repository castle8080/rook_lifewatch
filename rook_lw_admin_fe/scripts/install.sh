#!/bin/sh
set -e

rm -rf ../dist/www/admin/rook_lw_admin_fe-* ../dist/www/admin/index.html
mkdir -p ../dist/www/admin
cp -vf dist/* ../dist/www/admin
