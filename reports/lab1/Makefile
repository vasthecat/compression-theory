.PHONY: clean all

all:
	@echo "Использование:"
	@echo "make entr -> Запуск entr, пересобирающий документ пи изменениях"
	@echo "make clean -> Удаление сгенерированных файлов"

entr:
	sh -c "echo *.tex | entr latexmk -pdf -f -shell-escape lab*.tex"

clean:
	rm -rf _minted-* *.aux *.dvi *.fdb_latexmk *.fls *.log *.dvi
