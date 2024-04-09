FROM ubuntu:20.04
RUN ln -sf /usr/share/zoneinfo/Asia/Shanghai /etc/localtime && echo 'Asia/Shanghai' >/etc/timezone
RUN apt-get update && apt install -y libssl-dev ca-certificates wget && apt-get clean

ADD swan-check-aleo-proof /mnt/init/cmd
RUN chmod +x /mnt/init/cmd

CMD [ "/mnt/init/cmd", "-p", "8080" ]