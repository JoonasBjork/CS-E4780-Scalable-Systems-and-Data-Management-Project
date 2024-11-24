FROM denoland/deno:alpine-1.46.3

WORKDIR /app

COPY deps.js .

RUN deno cache deps.js

COPY . .

CMD [ "run", "--unstable", "--watch", "--allow-net", "--allow-read", "--allow-env", "--v8-flags=--expose-gc", "app.js" ]