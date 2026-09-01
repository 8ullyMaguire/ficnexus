FROM python:3.12-slim
WORKDIR /srv
COPY *.py *.yaml README.md /srv/
RUN pip install --no-cache-dir flask numpy scipy \
    && (pip install --no-cache-dir lightfm implicit vowpalwabbit || true)
EXPOSE 8300
CMD ["python", "lightfm_recipe.py"]
