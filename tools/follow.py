import subprocess
import re
import time

while True:
    try:
        pid = subprocess.check_output(['pgrep', 'p4']).decode("utf-8") 
    except:
        time.sleep(0.5)
        continue

    process = subprocess.Popen(f"strace -p{pid} -s9999 -e write", shell=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)

    current_line = ""
    while True:
        output = process.stdout.readline()
        if not output and process.poll() is not None:
            break
        if output and "write" in output.decode('utf-8'):
            line = output.decode('utf-8').strip()
            line = re.sub(r"write\([0-9], \"", "", line)
            line = re.sub(r"\", [0-9]+\).*", "", line)
            
            line = line.replace("\\n", "\n")
            print(line, end='')
