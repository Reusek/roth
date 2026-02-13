LATEX=pdflatex
BIBER=biber
MAKEGLOSSARIES=makeglossaries
MAKEINDEX=makeindex

.PHONY: all kidiplom clean dist

all: kidiplom.pdf

kidiplom: kidiplom.pdf

%.pdf: %.tex
	@$(LATEX) $<
	@$(LATEX) $<
	@$(BIBER) $*
	@$(MAKEGLOSSARIES) $*
	@$(MAKEINDEX) $*.idx
	@$(LATEX) $<
	@$(LATEX) $<

clean:
	@rm -v -f *.glsdefs *.bcf *.lo* *.aux *.ind *.idx *.ilg *.toc *.acn *.run.xml *-blx.bib *.ist *.glo  *.blg *.bbl  *.gls *.glg *.alg *.acr

dist:
	@mkdir -p kidiplom
	@cp -r -f kidiplom.pdf kidiplom.tex kidiplom.cls kibase.sty graphics *bib *bx Makefile README.txt LICENSE kidiplom/
	@rm -f kidiplom.zip
	@zip -r kidiplom.zip kidiplom
	@rm -r -f kidiplom
