#!/bin/bash
# Build docs and copy to frontend static + build directories
set -e
cd "$(dirname "$0")"
echo "Building mdbook docs..."
mdbook build
# mdbook 0.5.4 writes HTML to book/html and EPUB to book/epub
echo "Copying to frontend/static/docs..."
rm -rf ../frontend/static/docs
cp -r book/html ../frontend/static/docs
echo "Copying to frontend/build/docs..."
rm -rf ../frontend/build/docs
cp -r book/html ../frontend/build/docs
echo "Copying docs EPUB..."
cp "book/epub/FicHub Docs.epub" ./FicHub_Docs.epub
echo "Done! Docs are in frontend/static/docs/, frontend/build/docs/, and ./FicHub_Docs.epub"
