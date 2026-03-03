FROM devopsfaith/krakend:2.7 AS builder
WORKDIR /etc/krakend
COPY krakend.json krakend.json
RUN ["krakend", "check", "-c", "/etc/krakend/krakend.json"]

FROM devopsfaith/krakend:2.7
WORKDIR /etc/krakend
COPY --from=builder /etc/krakend/krakend.json /etc/krakend/krakend.json
EXPOSE 8080
CMD ["run", "-c", "/etc/krakend/krakend.json"]
