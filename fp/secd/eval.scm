(letrec
;; what:
(
(unzip (lambda (ps)
    (letrec
      ((unzipt
         (lambda (pairs z1 z2)
           (if (null? pairs)
             (list z1 z2)
             (let ((pair (car pairs))
                   (rest (cdr pairs)))
               (let ((p1 (car pair))
                     (p2 (cadr pair)))
                 (unzipt rest (append z1 (list p1)) (append z2 (list p2)))))))))
      (unzipt ps '() '()))))

(compile-bindings
  (lambda (bs)
    (if (null? bs) '(LDC ())
        (append (compile-bindings (cdr bs))
                (compile (car bs))
                '(CONS)))))

(compile-form (lambda (f)
  (let ((hd (car f))
        (tl (cdr f)))
    (cond
      ((eq? hd 'quote)
        (list 'LDC (car tl)))
      ((eq? hd '+)
        (append (compile (car tl)) (compile (cadr tl)) '(ADD)))
      ((eq? hd '*)
        (append (compile (car tl)) (compile (cadr tl)) '(MUL)))
      ((eq? hd 'atom?)
        (append (compile (car tl)) '(ATOM)))
      ((eq? hd 'car)
        (append (compile (car tl)) '(CAR)))
      ((eq? hd 'cdr)
        (append (compile (car tl)) '(CDR)))
      ((eq? hd 'cons)
        (append (compile (cadr tl)) (compile (car tl)) '(CONS)))
      ((eq? hd 'eq? )
        (append (compile (car tl)) (compile (cadr tl)) '(EQ)))
      ((eq? hd 'if )
        (let ((condc (compile (car tl)))
              (thenb (append (compile (cadr tl)) '(JOIN)))
              (elseb (append (compile (caddr tl)) '(JOIN))))
          (append condc '(SEL) (list thenb) (list elseb))))
      ((eq? hd 'lambda)
        (let ((args (car tl))
              (body (append (compile (cadr tl)) '(RTN))))
          (list 'LDF (list args body))))
      ((eq? hd 'let)
        (let ((bindings (unzip (car tl)))
              (body (cadr tl)))
          (let ((args (car bindings))
                (exprs (cadr bindings)))
            (append (compile-bindings exprs)
                    (list 'LDF (list args (append (compile body) '(RTN))))
                    '(AP)))))
      ((eq? hd 'letrec)
        (let ((bindings (unzip (car tl)))
              (body (cadr tl)))
          (let ((args (car bindings))
                (exprs (cadr bindings)))
              (append '(DUM)
                      (compile-bindings exprs)
                      (list 'LDF (list args (append (compile body) '(RTN))))
                      '(RAP)))))
      ;((eq? hd 'begin)
      ((eq? hd 'display)
        (append (compile (car tl)) '(PRINT)))
      ((eq? hd 'read)
        '(READ))
      ((eq? hd 'quit)
        '(STOP))
      (else
        (append (compile-bindings tl)
                (list 'LD hd)
                '(AP)))
    ))))

(compile (lambda (s)
  (cond
    ((symbol? s) (list 'LD s))
    ((number? s) (list 'LDC s))
    (else (compile-form s)))))

(loop (lambda (body)
   (begin
     (body) (loop body))))
)

;; <let> in
(loop (lambda ()
    (display (compile (read)))
    (newline)))
)
