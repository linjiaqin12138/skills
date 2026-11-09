import urllib.request, urllib.parse, sys, os, time
def gen(prompt, fn, w=1080, h=1920, seed=None, model="flux"):
    q = urllib.parse.urlencode({"width":w,"height":h,"model":model,"nologo":"true","seed":seed} if seed else {"width":w,"height":h,"model":model,"nologo":"true"})
    url = f"https://image.pollinations.ai/prompt/{urllib.parse.quote(prompt)}?{q}"
    for attempt in range(3):
        try:
            req=urllib.request.Request(url, headers={"User-Agent":"Mozilla/5.0"})
            d=urllib.request.urlopen(req,timeout=180).read()
            if len(d)>10000:
                open(fn,"wb").write(d); return len(d)
        except Exception as e:
            print("retry",attempt,e)
        time.sleep(5)
    return 0
if __name__=="__main__":
    p=sys.argv[1]; fn=sys.argv[2]; seed=int(sys.argv[3]) if len(sys.argv)>3 else None
    print(fn, gen(p,fn,seed=seed))
