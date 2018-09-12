FROM python:3.6.3
  
COPY ./requirements.txt /requirements.txt
RUN pip install --no-cache-dir -r /requirements.txt

COPY . /client
WORKDIR /client

COPY --from=roboserver /server/target/debug/librobodrivers.so /client/lib/

ENTRYPOINT ["./roboclient.py", "--lib_dir", "lib"]

# default arguments, can be overriden from the "docker run" command-line
CMD ["--help"]
