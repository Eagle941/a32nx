@echo off

set image="ghcr.io/flybywiresim/dev-env@sha256:52405005788639dfc7f87b80dc8b088c5f92d0c6b6dd769fa23442463085d78c"
set envfile="%cd%\.env"

if not exist %envfile% (
    type nul > %envfile%
)

docker image inspect %image% 1> nul || docker system prune --filter label=flybywiresim=true -f
docker run --rm -it -v "%cd%:/external" --env-file %envfile% %image% %*
