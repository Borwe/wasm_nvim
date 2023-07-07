import msgpack as mp
import subprocess as proc
import json

def main():
    api_info = proc.check_output(["nvim", "--api-info"])
    nvim_api = mp.unpackb(api_info, strict_map_key=False)
    with open("./api_info_v0.9.1.json", "w") as f:
        json.dump(nvim_api, fp=f, indent=4)

if __name__=="__main__":
    main()
