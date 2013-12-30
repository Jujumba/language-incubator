{-
-   See: http://matt.might.net/articles/cek-machines/
-}
import Data.Maybe (fromJust)
import qualified Data.Map as M

type Var = String
data Val 
    = VInt Int
    | VClo Closure
    | VLam Lam
  deriving (Show, Read, Eq)

data Lam = Var :=> Exp 
  deriving (Read, Show, Eq)

data Exp
    = MRef Var
    | MVal Val
    | MAp Exp Exp
  deriving (Read, Show, Eq)

type Program = Exp
type State   = (Exp, Env, Kont)
data Closure = Clo Lam Env 
  deriving (Show, Read, Eq)
data Kont
    = KStart
    | KEmpty
    | KHoleFun (Exp, Env) Kont
    | KHoleArg Val Kont
  deriving (Read, Show)

type Env = M.Map Var Val

{-
 - Environment
 -}
emptyEnv :: Env
emptyEnv = M.empty

extendEnv :: (Var, Val) -> Env -> Env
extendEnv (var, val) e = M.insert var val e

lookupEnv :: Var -> Env -> Val
lookupEnv v e = 
    case M.lookup v e of
      Just r -> r
      Nothing -> error $ "Lookup of '" ++ v ++ "' failed in E{" ++ show e ++ "}"

{-
 - CEK machine
 -}
step :: State -> State
step (MRef x,         e,  k)                  = (MVal (lookupEnv x e),    e, k)
step (MAp m1 m2,      e,  k)                  = (m1,                      e, KHoleFun (m2, e) k)
step (MVal (VLam lam), e,  k)                 = (MVal (VClo (Clo lam e)), e, k)
step (MVal w,      e1, KHoleFun (m, e2) k)    = (m, e2, KHoleArg w k)
step (MVal w,      e1, KHoleArg (VClo (Clo (x :=> m) e2)) k) 
    = (m, extendEnv (x, w) e2, k)
--step (w,     e, k) = (w, e, KEmpty)

run :: State -> State
run (exp, env, KEmpty) = (exp, env, KEmpty)
run (exp, env, KStart) = run $ step (exp, env, KEmpty)
run (exp, env, k)      = run $ step (exp, env, k)

evaluate :: Program -> Exp
evaluate exp = 
    case run (exp, emptyEnv, KStart) of
      (result, _, KEmpty) -> result
      st -> error $ "Program exited with invalid state " ++ show st

{-
 - for GHCi
 -}
lam = MVal . VLam
int = MVal . VInt
ref = MRef
test_id = lam ("x" :=> ref "x")
test_int = int 42
test1 = MAp test_id test_int
test2 = MAp (MAp (lam ("x" :=> (lam ("y" :=> (ref "x"))))) (int 1)) (int 2)

main = do
    exp <- readLn :: IO Program
    print . evaluate $ exp
