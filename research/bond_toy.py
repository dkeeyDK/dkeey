"""Toy check of the DK 'bond' primitives. NOT secure (small group, no ZK, no network). Only checks the math behaves as claimed."""
import hashlib, random, os, json
rnd = random.SystemRandom()
CACHE = "/tmp/dkeey_safeprime.json"

def is_prime(n, k=24):
    if n < 4: return n in (2, 3)
    if n % 2 == 0: return False
    for sp in (3,5,7,11,13,17,19,23,29,31,37):
        if n % sp == 0: return n == sp
    d, s = n - 1, 0
    while d % 2 == 0: d //= 2; s += 1
    for _ in range(k):
        a = rnd.randrange(2, n - 1); x = pow(a, d, n)
        if x in (1, n - 1): continue
        for _ in range(s - 1):
            x = x * x % n
            if x == n - 1: break
        else: return False
    return True

def safe_prime(bits=256):
    if os.path.exists(CACHE):
        d = json.load(open(CACHE))
        if d["bits"] == bits: return d["p"]
    while True:
        q = rnd.getrandbits(bits - 1) | (1 << (bits - 2)) | 1
        if is_prime(q) and is_prime(2 * q + 1):
            json.dump({"bits": bits, "p": 2 * q + 1}, open(CACHE, "w")); return 2 * q + 1

P = safe_prime(256); Q = (P - 1) // 2          # subgroup of squares has prime order Q
def hash_to_group(label):                       # nothing-up-my-sleeve generator
    x = int.from_bytes(hashlib.sha256(label.encode()).digest(), "big") % P
    return pow(x, 2, P)
G, H = hash_to_group("DK/G"), hash_to_group("DK/H")

def commit(v, r): return pow(G, v % Q, P) * pow(H, r % Q, P) % P
def rerandomize(c, dr): return c * pow(H, dr % Q, P) % P      # same value, new look

# Shamir secret sharing over Z_Q
def share(secret, t, n):
    co = [secret] + [rnd.randrange(Q) for _ in range(t - 1)]
    return {i: sum(c * pow(i, j, Q) for j, c in enumerate(co)) % Q for i in range(1, n + 1)}
def lagrange0(ids, i):
    num = den = 1
    for j in ids:
        if j != i: num = num * (-j) % Q; den = den * (i - j) % Q
    return num * pow(den, -1, Q) % Q
def reconstruct(sh):
    ids = list(sh); return sum(sh[i] * lagrange0(ids, i) for i in ids) % Q

# Threshold PRF: tag(x) = H(x)^k, k secret-shared; each seat returns H(x)^{k_i}
def hx(x): return hash_to_group("TAG/" + x)
def seat_partial(x, ki): return pow(hx(x), ki, P)
def combine(partials):
    ids = list(partials); out = 1
    for i in ids: out = out * pow(partials[i], lagrange0(ids, i), P) % P
    return out

def run():
    ok = True
    def chk(name, cond):
        nonlocal ok; ok &= bool(cond); print(("PASS " if cond else "FAIL ") + name)
    print(f"group: {P.bit_length()}-bit safe prime (toy size)")
    # 1) commitments: re-randomization keeps value, changes look; sums are conserved (homomorphic)
    v, r = 120, rnd.randrange(Q); c = commit(v, r); dr = rnd.randrange(Q); c2 = rerandomize(c, dr)
    chk("re-randomized commitment differs from the original", c2 != c)
    chk("re-randomized commitment opens to the SAME value with r+dr", c2 == commit(v, r + dr))
    a, b = commit(70, 5), commit(50, 9)
    chk("homomorphic: commit(70)+commit(50) opens as 120 (no inflation)", a * b % P == commit(120, 14))
    chk("a wrong value does not open the commitment", c != commit(v + 1, r))
    # unlinkability sanity: 1000 re-randomizations of one value look unrelated (no collisions)
    seen = {rerandomize(c, rnd.randrange(Q)) for _ in range(1000)}
    chk("1000 re-randomizations are all distinct", len(seen) == 1000)
    # 2) threshold secret: 6-of-8
    k = rnd.randrange(1, Q); sh = share(k, 6, 8)
    chk("any 6 shares reconstruct the key", reconstruct({i: sh[i] for i in (1, 3, 4, 6, 7, 8)}) == k)
    five = {i: sh[i] for i in (1, 2, 3, 4, 5)}
    wrong = reconstruct(five)
    chk("5 shares reconstruct the WRONG value", wrong != k)
    # information-theoretic check: 5 shares are consistent with ANY chosen secret (extend with a fake 6th point)
    fake = rnd.randrange(Q); ids = list(five) + [0]; pts = dict(five); pts[0] = fake
    def interp(pts, x):
        s = 0
        for i in pts:
            num = den = 1
            for j in pts:
                if j != i: num = num * (x - j) % Q; den = den * (i - j) % Q
            s += pts[i] * num * pow(den, -1, Q)
        return s % Q
    chk("5 shares are consistent with an arbitrary fake secret (no information)", interp(pts, 0) == fake)
    # 3) threshold PRF: tag from 6 partials equals the tag from the full key; 5 partials do not
    x = "unit-42"; full = pow(hx(x), k, P)
    part6 = {i: seat_partial(x, sh[i]) for i in (2, 3, 4, 5, 7, 8)}
    part5 = {i: seat_partial(x, sh[i]) for i in (1, 2, 3, 4, 5)}
    chk("6 partial results combine to the real tag", combine(part6) == full)
    chk("5 partial results do NOT give the real tag", combine(part5) != full)
    chk("no single seat's partial equals the tag", all(v != full for v in part6.values()))
    # 4) nullifier: same unit gives the same nullifier (double spend detected); different units differ
    nul = lambda u: hashlib.sha256(str(combine({i: seat_partial(u, sh[i]) for i in (1, 2, 3, 4, 5, 6)})).encode()).hexdigest()
    spent = set(); n1 = nul("unit-42"); spent.add(n1)
    chk("spending the same unit twice is caught", nul("unit-42") in spent)
    chk("a different unit has a different nullifier", nul("unit-43") not in spent)
    print("ALL PASS" if ok else "SOME FAILED")
if __name__ == "__main__": run()
