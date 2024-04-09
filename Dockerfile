FROM ubuntu:20.04
RUN ln -sf /usr/share/zoneinfo/Asia/Shanghai /etc/localtime && echo 'Asia/Shanghai' >/etc/timezone

ADD swan-aleo-proof-check /mnt/init/cmd
RUN chmod +x /mnt/init/cmd
