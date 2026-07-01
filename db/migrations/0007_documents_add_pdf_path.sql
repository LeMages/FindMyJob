-- Store the compiled PDF path for reference .typ CVs.
-- Uploaded PDFs use file_path directly (pdf_path stays NULL).
ALTER TABLE documents ADD COLUMN pdf_path TEXT;
